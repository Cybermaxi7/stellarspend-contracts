use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Events as _},
    Address, Env, Map, String, Vec, U256,
};

#[path = "../contracts/errors.rs"]
mod errors;

#[path = "../contracts/lib.rs"]
mod lib;

use errors::{
    ErrorCategory, ErrorContext, ErrorDocumentation, ErrorHelpers, ErrorSeverity, RetryStrategy,
    StellarSpendError,
};
use lib::{testing, ContractUtils, EventEmit};

fn setup_error_test() -> (Env, Address) {
    testing::setup_test_env()
}

#[test]
fn test_error_code_conversion() {
    let env = Env::default();

    // Test valid error code conversion
    let error = ErrorDocumentation::code_to_error(1100);
    assert!(error.is_some());
    assert_eq!(error.unwrap(), StellarSpendError::Unauthorized);

    // Test invalid error code
    let invalid_error = ErrorDocumentation::code_to_error(9999);
    assert!(invalid_error.is_none());
}

#[test]
fn test_error_categories() {
    let unauthorized = StellarSpendError::Unauthorized;
    assert_eq!(unauthorized.category(), ErrorCategory::Authorization);

    let insufficient_balance = StellarSpendError::InsufficientBalance;
    assert_eq!(insufficient_balance.category(), ErrorCategory::Balance);

    let overflow = StellarSpendError::Overflow;
    assert_eq!(overflow.category(), ErrorCategory::Arithmetic);

    let not_initialized = StellarSpendError::NotInitialized;
    assert_eq!(not_initialized.category(), ErrorCategory::Initialization);
}

#[test]
fn test_error_severity() {
    let security_violation = StellarSpendError::SecurityViolation;
    assert_eq!(security_violation.severity(), ErrorSeverity::Critical);

    let unauthorized = StellarSpendError::Unauthorized;
    assert_eq!(unauthorized.severity(), ErrorSeverity::High);

    let invalid_input = StellarSpendError::InvalidInput;
    assert_eq!(invalid_input.severity(), ErrorSeverity::Medium);

    let not_found = StellarSpendError::NotFound;
    assert_eq!(not_found.severity(), ErrorSeverity::Low);
}

#[test]
fn test_error_recoverability() {
    let insufficient_balance = StellarSpendError::InsufficientBalance;
    assert!(insufficient_balance.is_recoverable());

    let rate_limit = StellarSpendError::RateLimitExceeded;
    assert!(rate_limit.is_recoverable());

    let security_violation = StellarSpendError::SecurityViolation;
    assert!(!security_violation.is_recoverable());

    let unauthorized = StellarSpendError::Unauthorized;
    assert!(!unauthorized.is_recoverable());
}

#[test]
fn test_retry_delays() {
    let rate_limit = StellarSpendError::RateLimitExceeded;
    assert_eq!(rate_limit.retry_delay(), Some(60));

    let network_error = StellarSpendError::NetworkError;
    assert_eq!(network_error.retry_delay(), Some(30));

    let oracle_unavailable = StellarSpendError::OracleUnavailable;
    assert_eq!(oracle_unavailable.retry_delay(), Some(120));

    let maintenance_mode = StellarSpendError::MaintenanceMode;
    assert_eq!(maintenance_mode.retry_delay(), Some(300));

    let invalid_input = StellarSpendError::InvalidInput;
    assert_eq!(invalid_input.retry_delay(), None);
}

#[test]
fn test_error_documentation() {
    let env = Env::default();

    // Test documentation for known error
    let doc = ErrorDocumentation::get_documentation(&env, 1100);
    assert!(doc.is_some());

    let documentation = doc.unwrap();
    assert_eq!(documentation.code, 1100);
    assert_eq!(documentation.category, ErrorCategory::Authorization);
    assert_eq!(documentation.severity, ErrorSeverity::High);
    assert!(documentation.recoverable);
    assert_eq!(documentation.retry_delay, None);

    // Test documentation for unknown error
    let unknown_doc = ErrorDocumentation::get_documentation(&env, 9999);
    assert!(unknown_doc.is_none());
}

#[test]
fn test_error_context_creation() {
    let (env, _) = setup_error_test();

    let mut parameters = Vec::new(&env);
    parameters.push_back(String::from_str(&env, "param1"));
    parameters.push_back(String::from_str(&env, "param2"));

    let mut additional_info = Map::new(&env);
    additional_info.set(
        String::from_str(&env, "key1"),
        String::from_str(&env, "value1"),
    );

    let context = ErrorHelpers::create_context(
        &env,
        1100,
        "TestContract",
        "test_function",
        parameters.clone(),
        additional_info.clone(),
    );

    assert_eq!(context.error_code, 1100);
    assert_eq!(
        context.contract_name,
        String::from_str(&env, "TestContract")
    );
    assert_eq!(
        context.function_name,
        String::from_str(&env, "test_function")
    );
    assert_eq!(context.parameters, parameters);
    assert_eq!(context.additional_info, additional_info);
}

#[test]
fn test_error_logging_decisions() {
    // Critical errors should be logged
    assert!(ErrorHelpers::should_log(2000)); // SecurityViolation
    assert!(ErrorHelpers::should_log(2100)); // SystemError

    // High severity errors should be logged
    assert!(ErrorHelpers::should_log(1100)); // Unauthorized
    assert!(ErrorHelpers::should_log(1400)); // InsufficientBalance
    assert!(ErrorHelpers::should_log(1600)); // Overflow

    // Medium severity errors should be logged
    assert!(ErrorHelpers::should_log(1200)); // InvalidInput
    assert!(ErrorHelpers::should_log(1500)); // LimitExceeded

    // Low severity errors should not be logged
    assert!(!ErrorHelpers::should_log(1300)); // NotFound
    assert!(!ErrorHelpers::should_log(1304)); // Expired
}

#[test]
fn test_retry_strategies() {
    // Immediate retry for transient errors
    assert_eq!(
        ErrorHelpers::retry_strategy(1800), // NetworkError
        RetryStrategy::Immediate
    );
    assert_eq!(
        ErrorHelpers::retry_strategy(1802), // OracleUnavailable
        RetryStrategy::Immediate
    );

    // Exponential backoff for rate limits
    assert_eq!(
        ErrorHelpers::retry_strategy(1503), // RateLimitExceeded
        RetryStrategy::ExponentialBackoff
    );

    // Fixed delay for maintenance
    assert_eq!(
        ErrorHelpers::retry_strategy(2103), // MaintenanceMode
        RetryStrategy::FixedDelay
    );

    // No retry for permanent errors
    assert_eq!(
        ErrorHelpers::retry_strategy(1100), // Unauthorized
        RetryStrategy::NoRetry
    );
    assert_eq!(
        ErrorHelpers::retry_strategy(2000), // SecurityViolation
        RetryStrategy::NoRetry
    );
    assert_eq!(
        ErrorHelpers::retry_strategy(1400), // InsufficientBalance
        RetryStrategy::NoRetry
    );

    // Default to exponential backoff
    assert_eq!(
        ErrorHelpers::retry_strategy(1200), // InvalidInput
        RetryStrategy::ExponentialBackoff
    );
}

#[test]
fn test_standardized_error_macro() {
    let (env, _) = setup_error_test();

    // Test that std_error macro compiles and works
    let result = std::panic::catch_unwind(|| {
        std_error!(&env, StellarSpendError::InvalidInput);
    });

    assert!(result.is_err());
}

#[test]
fn test_validation_macro() {
    let (env, _) = setup_error_test();

    // Test successful validation
    let result = std::panic::catch_unwind(|| {
        validate!(&env, 5 > 3, StellarSpendError::InvalidInput);
    });
    assert!(result.is_ok());

    // Test failed validation
    let result2 = std::panic::catch_unwind(|| {
        validate!(&env, 1 > 3, StellarSpendError::InvalidInput);
    });
    assert!(result2.is_err());
}

#[test]
fn test_require_auth_macro() {
    let (env, admin) = setup_error_test();
    let user = Address::generate(&env);

    // Test successful auth
    let result = std::panic::catch_unwind(|| {
        require_auth!(&env, &admin, &admin);
    });
    assert!(result.is_ok());

    // Test failed auth
    let result2 = std::panic::catch_unwind(|| {
        require_auth!(&env, &user, &admin);
    });
    assert!(result2.is_err());
}

#[test]
fn test_validate_amount_macro() {
    let (env, _) = setup_error_test();

    // Test valid amount
    let result = std::panic::catch_unwind(|| {
        validate_amount!(&env, 100i128);
    });
    assert!(result.is_ok());

    // Test invalid amount (zero)
    let result2 = std::panic::catch_unwind(|| {
        validate_amount!(&env, 0i128);
    });
    assert!(result2.is_err());

    // Test amount too large
    let result3 = std::panic::catch_unwind(|| {
        validate_amount!(&env, i128::MAX);
    });
    assert!(result3.is_err());

    // Test amount with min/max bounds
    let result4 = std::panic::catch_unwind(|| {
        validate_amount!(&env, 50i128, 10i128, 100i128);
    });
    assert!(result4.is_ok());

    let result5 = std::panic::catch_unwind(|| {
        validate_amount!(&env, 5i128, 10i128, 100i128);
    });
    assert!(result5.is_err());
}

#[test]
fn test_validate_address_macro() {
    let (env, _) = setup_error_test();

    let valid_address = Address::generate(&env);
    let zero_address = Address::from_contract_id(&env);

    // Test valid address
    let result = std::panic::catch_unwind(|| {
        validate_address!(&env, &valid_address);
    });
    assert!(result.is_ok());

    // Test zero address
    let result2 = std::panic::catch_unwind(|| {
        validate_address!(&env, &zero_address);
    });
    assert!(result2.is_err());
}

#[test]
fn test_safe_arithmetic_macros() {
    let (env, _) = setup_error_test();

    // Test safe addition
    let result = safe_add!(&env, 100i128, 50i128);
    assert_eq!(result, 150i128);

    // Test safe subtraction
    let result2 = safe_sub!(&env, 100i128, 50i128);
    assert_eq!(result2, 50i128);

    // Test safe multiplication
    let result3 = safe_mul!(&env, 10i128, 5i128);
    assert_eq!(result3, 50i128);

    // Test safe division
    let result4 = safe_div!(&env, 100i128, 5i128);
    assert_eq!(result4, 20i128);

    // Test division by zero
    let result5 = std::panic::catch_unwind(|| {
        safe_div!(&env, 100i128, 0i128);
    });
    assert!(result5.is_err());
}

#[test]
fn test_contract_utils() {
    let (env, admin) = setup_error_test();

    // Test admin storage (should fail since not initialized)
    let result = std::panic::catch_unwind(|| {
        ContractUtils::get_admin(&env);
    });
    assert!(result.is_err());

    // Test initialization check
    assert!(!ContractUtils::is_initialized(&env));

    // Test timestamp validation
    let timestamp = ContractUtils::get_timestamp(&env);
    assert!(timestamp > 0);

    // Test transaction ID generation
    let tx_id1 = ContractUtils::generate_transaction_id(&env);
    let tx_id2 = ContractUtils::generate_transaction_id(&env);
    assert_ne!(tx_id1, tx_id2);
}

#[test]
fn test_event_emission() {
    let (env, admin) = setup_error_test();
    let user = Address::generate(&env);

    let mut parameters = Vec::new(&env);
    parameters.push_back(String::from_str(&env, "param1"));

    // Test operation started event
    EventEmit::operation_started(&env, "test_operation", &user, parameters.clone());

    // Test operation completed event
    EventEmit::operation_completed(&env, "test_operation", &user, "success");

    // Test operation failed event
    EventEmit::operation_failed(
        &env,
        "test_operation",
        &user,
        StellarSpendError::InvalidInput,
    );

    // Check events were emitted
    let events = env.events().all();
    assert!(events.len() >= 3);
}

#[test]
fn test_rate_limiting() {
    let (env, user) = setup_error_test();
    let operation = "test_operation";

    // First operation should succeed
    let result1 = ContractUtils::check_rate_limit(&env, &user, operation, 2, 60);
    assert!(result1.is_ok());

    // Second operation should succeed
    let result2 = ContractUtils::check_rate_limit(&env, &user, operation, 2, 60);
    assert!(result2.is_ok());

    // Third operation should fail (rate limit exceeded)
    let result3 = ContractUtils::check_rate_limit(&env, &user, operation, 2, 60);
    assert!(result3.is_err());
    assert_eq!(result3.unwrap_err(), StellarSpendError::RateLimitExceeded);
}

#[test]
fn test_all_error_codes_documented() {
    let env = Env::default();

    // Test that all error codes have documentation
    let error_codes = vec![
        1000, 1001, 1002, // Initialization
        1100, 1101, 1102, 1103, 1104, // Authorization
        1200, 1201, 1202, 1203, 1204, 1205, 1206, 1207, // Validation
        1300, 1301, 1302, 1303, 1304, 1305, 1306, // State
        1400, 1401, 1402, 1403, 1404, 1405, 1406, 1407, // Balance
        1500, 1501, 1502, 1503, 1504, 1505, // Limit
        1600, 1601, 1602, 1603, // Arithmetic
        1700, 1701, 1702, 1703, 1704, // Storage
        1800, 1801, 1802, 1803, // External
        1900, 1901, 1902, 1903, 1904, // Business Logic
        2000, 2001, 2002, 2003, 2004, // Security
        2100, 2101, 2102, 2103, 2104, // System
    ];

    for code in error_codes {
        let doc = ErrorDocumentation::get_documentation(&env, code);
        assert!(
            doc.is_some(),
            "Missing documentation for error code {}",
            code
        );

        let documentation = doc.unwrap();
        assert!(
            !documentation.name.is_empty(),
            "Empty name for error code {}",
            code
        );
        assert!(
            !documentation.description.is_empty(),
            "Empty description for error code {}",
            code
        );
        assert!(
            !documentation.causes.is_empty(),
            "No causes for error code {}",
            code
        );
        assert!(
            !documentation.solutions.is_empty(),
            "No solutions for error code {}",
            code
        );
    }
}

#[test]
fn test_error_severity_classification() {
    // Test that critical errors are properly classified
    let critical_errors = vec![
        StellarSpendError::SecurityViolation,
        StellarSpendError::SystemError,
        StellarSpendError::InternalError,
        StellarSpendError::CorruptedData,
    ];

    for error in critical_errors {
        assert_eq!(error.severity(), ErrorSeverity::Critical);
        assert!(!error.is_recoverable());
    }

    // Test that high severity errors are properly classified
    let high_errors = vec![
        StellarSpendError::Unauthorized,
        StellarSpendError::InsufficientBalance,
        StellarSpendError::Overflow,
        StellarSpendError::Underflow,
        StellarSpendError::StorageError,
    ];

    for error in high_errors {
        assert_eq!(error.severity(), ErrorSeverity::High);
    }

    // Test that low severity errors are properly classified
    let low_errors = vec![
        StellarSpendError::NotFound,
        StellarSpendError::Expired,
        StellarSpendError::NotActive,
        StellarSpendError::Paused,
    ];

    for error in low_errors {
        assert_eq!(error.severity(), ErrorSeverity::Low);
    }
}

#[test]
fn test_error_category_classification() {
    // Test initialization errors
    assert_eq!(
        StellarSpendError::NotInitialized.category(),
        ErrorCategory::Initialization
    );
    assert_eq!(
        StellarSpendError::AlreadyInitialized.category(),
        ErrorCategory::Initialization
    );

    // Test authorization errors
    assert_eq!(
        StellarSpendError::Unauthorized.category(),
        ErrorCategory::Authorization
    );
    assert_eq!(
        StellarSpendError::AdminRequired.category(),
        ErrorCategory::Authorization
    );

    // Test validation errors
    assert_eq!(
        StellarSpendError::InvalidInput.category(),
        ErrorCategory::Validation
    );
    assert_eq!(
        StellarSpendError::InvalidAmount.category(),
        ErrorCategory::Validation
    );

    // Test arithmetic errors
    assert_eq!(
        StellarSpendError::Overflow.category(),
        ErrorCategory::Arithmetic
    );
    assert_eq!(
        StellarSpendError::Underflow.category(),
        ErrorCategory::Arithmetic
    );
}

#[test]
fn test_comprehensive_error_scenario() {
    let (env, admin) = setup_error_test();
    let user = Address::generate(&env);

    // Simulate a complex error scenario
    let mut error_count = 0u32;

    // 1. Try to use uninitialized contract
    let result1 = std::panic::catch_unwind(|| {
        ContractUtils::get_admin(&env);
    });
    if result1.is_err() {
        error_count += 1;
        ContractUtils::emit_error_event(&env, StellarSpendError::NotInitialized, None);
    }

    // 2. Try invalid operation
    let result2 = std::panic::catch_unwind(|| {
        validate_amount!(&env, -100i128);
    });
    if result2.is_err() {
        error_count += 1;
        ContractUtils::emit_error_event(&env, StellarSpendError::NegativeAmount, None);
    }

    // 3. Try rate limited operation
    let result3 = ContractUtils::check_rate_limit(&env, &user, "test", 1, 60);
    if result3.is_ok() {
        // First operation succeeds
        let result4 = ContractUtils::check_rate_limit(&env, &user, "test", 1, 60);
        if result4.is_err() {
            error_count += 1;
            ContractUtils::emit_error_event(&env, StellarSpendError::RateLimitExceeded, None);
        }
    }

    // Verify error count and event emission
    assert!(error_count >= 2);

    let events = env.events().all();
    let error_events = events
        .iter()
        .filter(|event| {
            event.1.iter().any(|topic| {
                symbol_short!("error")
                    == soroban_sdk::Symbol::try_from_val(&env, &topic).unwrap_or(symbol_short!(""))
            })
        })
        .count();

    assert!(error_events >= 2);
}

#[test]
fn test_error_code_ranges() {
    // Test that error codes fall within expected ranges

    // Initialization errors (1000-1099)
    assert!(StellarSpendError::NotInitialized.code() >= 1000);
    assert!(StellarSpendError::NotInitialized.code() < 1100);

    // Authorization errors (1100-1199)
    assert!(StellarSpendError::Unauthorized.code() >= 1100);
    assert!(StellarSpendError::Unauthorized.code() < 1200);

    // Validation errors (1200-1299)
    assert!(StellarSpendError::InvalidInput.code() >= 1200);
    assert!(StellarSpendError::InvalidInput.code() < 1300);

    // System errors (2100-2199)
    assert!(StellarSpendError::SystemError.code() >= 2100);
    assert!(StellarSpendError::SystemError.code() < 2200);
}

#[test]
fn test_error_documentation_completeness() {
    let env = Env::default();

    // Test that all error categories are represented
    let categories = vec![
        ErrorCategory::Initialization,
        ErrorCategory::Authorization,
        ErrorCategory::Validation,
        ErrorCategory::State,
        ErrorCategory::Balance,
        ErrorCategory::Limit,
        ErrorCategory::Arithmetic,
        ErrorCategory::Storage,
        ErrorCategory::External,
        ErrorCategory::BusinessLogic,
        ErrorCategory::Security,
        ErrorCategory::System,
    ];

    for category in categories {
        let mut found = false;

        // Check that at least one error maps to each category
        let test_errors = vec![
            StellarSpendError::NotInitialized,
            StellarSpendError::Unauthorized,
            StellarSpendError::InvalidInput,
            StellarSpendError::NotFound,
            StellarSpendError::InsufficientBalance,
            StellarSpendError::LimitExceeded,
            StellarSpendError::Overflow,
            StellarSpendError::StorageError,
            StellarSpendError::NetworkError,
            StellarSpendError::TransactionFailed,
            StellarSpendError::SecurityViolation,
            StellarSpendError::SystemError,
        ];

        for error in test_errors {
            if error.category() == category {
                found = true;
                break;
            }
        }

        assert!(
            found,
            "Category {:?} not represented by any error",
            category
        );
    }
}
