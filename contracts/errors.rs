use soroban_sdk::{contracttype, Env, Map, String, Vec};

/// Comprehensive error standardization for StellarSpend contracts
///
/// This module provides a unified error handling system across all contracts
/// with standardized error codes, documentation mapping, and helper functions.

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[contracttype]
pub enum StellarSpendError {
    // === Initialization Errors (1000-1099) ===
    NotInitialized = 1000,
    AlreadyInitialized = 1001,
    InvalidInitialization = 1002,

    // === Authorization Errors (1100-1199) ===
    Unauthorized = 1100,
    InvalidSignature = 1101,
    InsufficientPermissions = 1102,
    AdminRequired = 1103,
    MinterRequired = 1104,

    // === Validation Errors (1200-1299) ===
    InvalidInput = 1200,
    InvalidAmount = 1201,
    InvalidAddress = 1202,
    InvalidTimestamp = 1203,
    InvalidParameter = 1204,
    InvalidConfiguration = 1205,
    InvalidTransaction = 1206,
    InvalidSignatureFormat = 1207,

    // === State Errors (1300-1399) ===
    NotFound = 1300,
    AlreadyExists = 1301,
    InvalidState = 1302,
    NotActive = 1303,
    Expired = 1304,
    Locked = 1305,
    Paused = 1306,

    // === Balance/Amount Errors (1400-1499) ===
    InsufficientBalance = 1400,
    InsufficientAllowance = 1401,
    InsufficientLiquidity = 1402,
    AmountExceedsLimit = 1403,
    NegativeAmount = 1404,
    ZeroAmount = 1405,
    AmountTooLarge = 1406,
    AmountTooSmall = 1407,

    // === Limit/Cap Errors (1500-1599) ===
    LimitExceeded = 1500,
    CapExceeded = 1501,
    QuotaExceeded = 1502,
    RateLimitExceeded = 1503,
    MaxUsersExceeded = 1504,
    MaxTransactionsExceeded = 1505,

    // === Arithmetic Errors (1600-1699) ===
    Overflow = 1600,
    Underflow = 1601,
    DivisionByZero = 1602,
    InvalidCalculation = 1603,

    // === Storage Errors (1700-1799) ===
    StorageError = 1700,
    CorruptedData = 1701,
    DataNotFound = 1702,
    WriteFailed = 1703,
    ReadFailed = 1704,

    // === Network/External Errors (1800-1899) ===
    NetworkError = 1800,
    ExternalCallFailed = 1801,
    OracleUnavailable = 1802,
    BridgeError = 1803,

    // === Business Logic Errors (1900-1999) ===
    TransactionFailed = 1900,
    ConditionNotMet = 1901,
    DeadlineExceeded = 1902,
    IncompatibleOperation = 1903,
    InvalidOperation = 1904,

    // === Security Errors (2000-2099) ===
    SecurityViolation = 2000,
    SuspiciousActivity = 2001,
    BlacklistedAddress = 2002,
    FrozenAccount = 2003,
    ComplianceViolation = 2004,

    // === System Errors (2100-2199) ===
    SystemError = 2100,
    InternalError = 2101,
    NotImplemented = 2102,
    MaintenanceMode = 2103,
    UpgradeRequired = 2104,
}

impl StellarSpendError {
    /// Get the error code as u32
    pub fn code(&self) -> u32 {
        *self as u32
    }

    /// Get the error category
    pub fn category(&self) -> ErrorCategory {
        match self {
            // Initialization
            StellarSpendError::NotInitialized
            | StellarSpendError::AlreadyInitialized
            | StellarSpendError::InvalidInitialization => ErrorCategory::Initialization,

            // Authorization
            StellarSpendError::Unauthorized
            | StellarSpendError::InvalidSignature
            | StellarSpendError::InsufficientPermissions
            | StellarSpendError::AdminRequired
            | StellarSpendError::MinterRequired => ErrorCategory::Authorization,

            // Validation
            StellarSpendError::InvalidInput
            | StellarSpendError::InvalidAmount
            | StellarSpendError::InvalidAddress
            | StellarSpendError::InvalidTimestamp
            | StellarSpendError::InvalidParameter
            | StellarSpendError::InvalidConfiguration
            | StellarSpendError::InvalidTransaction
            | StellarSpendError::InvalidSignatureFormat => ErrorCategory::Validation,

            // State
            StellarSpendError::NotFound
            | StellarSpendError::AlreadyExists
            | StellarSpendError::InvalidState
            | StellarSpendError::NotActive
            | StellarSpendError::Expired
            | StellarSpendError::Locked
            | StellarSpendError::Paused => ErrorCategory::State,

            // Balance/Amount
            StellarSpendError::InsufficientBalance
            | StellarSpendError::InsufficientAllowance
            | StellarSpendError::InsufficientLiquidity
            | StellarSpendError::AmountExceedsLimit
            | StellarSpendError::NegativeAmount
            | StellarSpendError::ZeroAmount
            | StellarSpendError::AmountTooLarge
            | StellarSpendError::AmountTooSmall => ErrorCategory::Balance,

            // Limit/Cap
            StellarSpendError::LimitExceeded
            | StellarSpendError::CapExceeded
            | StellarSpendError::QuotaExceeded
            | StellarSpendError::RateLimitExceeded
            | StellarSpendError::MaxUsersExceeded
            | StellarSpendError::MaxTransactionsExceeded => ErrorCategory::Limit,

            // Arithmetic
            StellarSpendError::Overflow
            | StellarSpendError::Underflow
            | StellarSpendError::DivisionByZero
            | StellarSpendError::InvalidCalculation => ErrorCategory::Arithmetic,

            // Storage
            StellarSpendError::StorageError
            | StellarSpendError::CorruptedData
            | StellarSpendError::DataNotFound
            | StellarSpendError::WriteFailed
            | StellarSpendError::ReadFailed => ErrorCategory::Storage,

            // Network/External
            StellarSpendError::NetworkError
            | StellarSpendError::ExternalCallFailed
            | StellarSpendError::OracleUnavailable
            | StellarSpendError::BridgeError => ErrorCategory::External,

            // Business Logic
            StellarSpendError::TransactionFailed
            | StellarSpendError::ConditionNotMet
            | StellarSpendError::DeadlineExceeded
            | StellarSpendError::IncompatibleOperation
            | StellarSpendError::InvalidOperation => ErrorCategory::BusinessLogic,

            // Security
            StellarSpendError::SecurityViolation
            | StellarSpendError::SuspiciousActivity
            | StellarSpendError::BlacklistedAddress
            | StellarSpendError::FrozenAccount
            | StellarSpendError::ComplianceViolation => ErrorCategory::Security,

            // System
            StellarSpendError::SystemError
            | StellarSpendError::InternalError
            | StellarSpendError::NotImplemented
            | StellarSpendError::MaintenanceMode
            | StellarSpendError::UpgradeRequired => ErrorCategory::System,
        }
    }

    /// Get the severity level of this error
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            // Critical errors that require immediate attention
            StellarSpendError::SecurityViolation
            | StellarSpendError::SystemError
            | StellarSpendError::InternalError
            | StellarSpendError::CorruptedData => ErrorSeverity::Critical,

            // High severity errors
            StellarSpendError::Unauthorized
            | StellarSpendError::InsufficientBalance
            | StellarSpendError::Overflow
            | StellarSpendError::Underflow
            | StellarSpendError::StorageError => ErrorSeverity::High,

            // Medium severity errors
            StellarSpendError::InvalidInput
            | StellarSpendError::InvalidAmount
            | StellarSpendError::LimitExceeded
            | StellarSpendError::CapExceeded
            | StellarSpendError::RateLimitExceeded => ErrorSeverity::Medium,

            // Low severity errors
            StellarSpendError::NotFound
            | StellarSpendError::Expired
            | StellarSpendError::NotActive
            | StellarSpendError::Paused => ErrorSeverity::Low,

            // Informational errors
            _ => ErrorSeverity::Info,
        }
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            // Recoverable errors
            StellarSpendError::InsufficientBalance
            | StellarSpendError::InsufficientAllowance
            | StellarSpendError::RateLimitExceeded
            | StellarSpendError::Paused
            | StellarSpendError::Expired
            | StellarSpendError::NotActive => true,

            // Non-recoverable errors
            StellarSpendError::SecurityViolation
            | StellarSpendError::SystemError
            | StellarSpendError::CorruptedData
            | StellarSpendError::Unauthorized => false,

            // Context dependent
            _ => false,
        }
    }

    /// Get suggested retry delay in seconds (if applicable)
    pub fn retry_delay(&self) -> Option<u64> {
        match self {
            StellarSpendError::RateLimitExceeded => Some(60),
            StellarSpendError::NetworkError => Some(30),
            StellarSpendError::OracleUnavailable => Some(120),
            StellarSpendError::MaintenanceMode => Some(300),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ErrorCategory {
    Initialization = 1000,
    Authorization = 1100,
    Validation = 1200,
    State = 1300,
    Balance = 1400,
    Limit = 1500,
    Arithmetic = 1600,
    Storage = 1700,
    External = 1800,
    BusinessLogic = 1900,
    Security = 2000,
    System = 2100,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ErrorSeverity {
    Critical = 4,
    High = 3,
    Medium = 2,
    Low = 1,
    Info = 0,
}

#[derive(Clone)]
#[contracttype]
pub struct ErrorDocumentation {
    pub code: u32,
    pub name: String,
    pub category: ErrorCategory,
    pub severity: ErrorSeverity,
    pub description: String,
    pub causes: Vec<String>,
    pub solutions: Vec<String>,
    pub recoverable: bool,
    pub retry_delay: Option<u64>,
}

#[derive(Clone)]
#[contracttype]
pub struct ErrorContext {
    pub error_code: u32,
    pub contract_name: String,
    pub function_name: String,
    pub parameters: Vec<String>,
    pub timestamp: u64,
    pub additional_info: Map<String, String>,
}

/// Error documentation and helper functions
pub struct ErrorDocumentation;

impl ErrorDocumentation {
    /// Get comprehensive documentation for an error code
    pub fn get_documentation(env: &Env, error_code: u32) -> Option<ErrorDocumentation> {
        let error = Self::code_to_error(error_code)?;

        Some(ErrorDocumentation {
            code: error_code,
            name: Self::error_name(&error),
            category: error.category(),
            severity: error.severity(),
            description: Self::error_description(&error),
            causes: Self::error_causes(&error),
            solutions: Self::error_solutions(&error),
            recoverable: error.is_recoverable(),
            retry_delay: error.retry_delay(),
        })
    }

    /// Convert error code to StellarSpendError enum
    pub fn code_to_error(code: u32) -> Option<StellarSpendError> {
        match code {
            // Initialization
            1000 => Some(StellarSpendError::NotInitialized),
            1001 => Some(StellarSpendError::AlreadyInitialized),
            1002 => Some(StellarSpendError::InvalidInitialization),

            // Authorization
            1100 => Some(StellarSpendError::Unauthorized),
            1101 => Some(StellarSpendError::InvalidSignature),
            1102 => Some(StellarSpendError::InsufficientPermissions),
            1103 => Some(StellarSpendError::AdminRequired),
            1104 => Some(StellarSpendError::MinterRequired),

            // Validation
            1200 => Some(StellarSpendError::InvalidInput),
            1201 => Some(StellarSpendError::InvalidAmount),
            1202 => Some(StellarSpendError::InvalidAddress),
            1203 => Some(StellarSpendError::InvalidTimestamp),
            1204 => Some(StellarSpendError::InvalidParameter),
            1205 => Some(StellarSpendError::InvalidConfiguration),
            1206 => Some(StellarSpendError::InvalidTransaction),
            1207 => Some(StellarSpendError::InvalidSignatureFormat),

            // State
            1300 => Some(StellarSpendError::NotFound),
            1301 => Some(StellarSpendError::AlreadyExists),
            1302 => Some(StellarSpendError::InvalidState),
            1303 => Some(StellarSpendError::NotActive),
            1304 => Some(StellarSpendError::Expired),
            1305 => Some(StellarSpendError::Locked),
            1306 => Some(StellarSpendError::Paused),

            // Balance/Amount
            1400 => Some(StellarSpendError::InsufficientBalance),
            1401 => Some(StellarSpendError::InsufficientAllowance),
            1402 => Some(StellarSpendError::InsufficientLiquidity),
            1403 => Some(StellarSpendError::AmountExceedsLimit),
            1404 => Some(StellarSpendError::NegativeAmount),
            1405 => Some(StellarSpendError::ZeroAmount),
            1406 => Some(StellarSpendError::AmountTooLarge),
            1407 => Some(StellarSpendError::AmountTooSmall),

            // Limit/Cap
            1500 => Some(StellarSpendError::LimitExceeded),
            1501 => Some(StellarSpendError::CapExceeded),
            1502 => Some(StellarSpendError::QuotaExceeded),
            1503 => Some(StellarSpendError::RateLimitExceeded),
            1504 => Some(StellarSpendError::MaxUsersExceeded),
            1505 => Some(StellarSpendError::MaxTransactionsExceeded),

            // Arithmetic
            1600 => Some(StellarSpendError::Overflow),
            1601 => Some(StellarSpendError::Underflow),
            1602 => Some(StellarSpendError::DivisionByZero),
            1603 => Some(StellarSpendError::InvalidCalculation),

            // Storage
            1700 => Some(StellarSpendError::StorageError),
            1701 => Some(StellarSpendError::CorruptedData),
            1702 => Some(StellarSpendError::DataNotFound),
            1703 => Some(StellarSpendError::WriteFailed),
            1704 => Some(StellarSpendError::ReadFailed),

            // Network/External
            1800 => Some(StellarSpendError::NetworkError),
            1801 => Some(StellarSpendError::ExternalCallFailed),
            1802 => Some(StellarSpendError::OracleUnavailable),
            1803 => Some(StellarSpendError::BridgeError),

            // Business Logic
            1900 => Some(StellarSpendError::TransactionFailed),
            1901 => Some(StellarSpendError::ConditionNotMet),
            1902 => Some(StellarSpendError::DeadlineExceeded),
            1903 => Some(StellarSpendError::IncompatibleOperation),
            1904 => Some(StellarSpendError::InvalidOperation),

            // Security
            2000 => Some(StellarSpendError::SecurityViolation),
            2001 => Some(StellarSpendError::SuspiciousActivity),
            2002 => Some(StellarSpendError::BlacklistedAddress),
            2003 => Some(StellarSpendError::FrozenAccount),
            2004 => Some(StellarSpendError::ComplianceViolation),

            // System
            2100 => Some(StellarSpendError::SystemError),
            2101 => Some(StellarSpendError::InternalError),
            2102 => Some(StellarSpendError::NotImplemented),
            2103 => Some(StellarSpendError::MaintenanceMode),
            2104 => Some(StellarSpendError::UpgradeRequired),

            _ => None,
        }
    }

    /// Get human-readable error name
    fn error_name(error: &StellarSpendError) -> String {
        match error {
            StellarSpendError::NotInitialized => "NotInitialized".into(),
            StellarSpendError::AlreadyInitialized => "AlreadyInitialized".into(),
            StellarSpendError::InvalidInitialization => "InvalidInitialization".into(),
            StellarSpendError::Unauthorized => "Unauthorized".into(),
            StellarSpendError::InvalidSignature => "InvalidSignature".into(),
            StellarSpendError::InsufficientPermissions => "InsufficientPermissions".into(),
            StellarSpendError::AdminRequired => "AdminRequired".into(),
            StellarSpendError::MinterRequired => "MinterRequired".into(),
            StellarSpendError::InvalidInput => "InvalidInput".into(),
            StellarSpendError::InvalidAmount => "InvalidAmount".into(),
            StellarSpendError::InvalidAddress => "InvalidAddress".into(),
            StellarSpendError::InvalidTimestamp => "InvalidTimestamp".into(),
            StellarSpendError::InvalidParameter => "InvalidParameter".into(),
            StellarSpendError::InvalidConfiguration => "InvalidConfiguration".into(),
            StellarSpendError::InvalidTransaction => "InvalidTransaction".into(),
            StellarSpendError::InvalidSignatureFormat => "InvalidSignatureFormat".into(),
            StellarSpendError::NotFound => "NotFound".into(),
            StellarSpendError::AlreadyExists => "AlreadyExists".into(),
            StellarSpendError::InvalidState => "InvalidState".into(),
            StellarSpendError::NotActive => "NotActive".into(),
            StellarSpendError::Expired => "Expired".into(),
            StellarSpendError::Locked => "Locked".into(),
            StellarSpendError::Paused => "Paused".into(),
            StellarSpendError::InsufficientBalance => "InsufficientBalance".into(),
            StellarSpendError::InsufficientAllowance => "InsufficientAllowance".into(),
            StellarSpendError::InsufficientLiquidity => "InsufficientLiquidity".into(),
            StellarSpendError::AmountExceedsLimit => "AmountExceedsLimit".into(),
            StellarSpendError::NegativeAmount => "NegativeAmount".into(),
            StellarSpendError::ZeroAmount => "ZeroAmount".into(),
            StellarSpendError::AmountTooLarge => "AmountTooLarge".into(),
            StellarSpendError::AmountTooSmall => "AmountTooSmall".into(),
            StellarSpendError::LimitExceeded => "LimitExceeded".into(),
            StellarSpendError::CapExceeded => "CapExceeded".into(),
            StellarSpendError::QuotaExceeded => "QuotaExceeded".into(),
            StellarSpendError::RateLimitExceeded => "RateLimitExceeded".into(),
            StellarSpendError::MaxUsersExceeded => "MaxUsersExceeded".into(),
            StellarSpendError::MaxTransactionsExceeded => "MaxTransactionsExceeded".into(),
            StellarSpendError::Overflow => "Overflow".into(),
            StellarSpendError::Underflow => "Underflow".into(),
            StellarSpendError::DivisionByZero => "DivisionByZero".into(),
            StellarSpendError::InvalidCalculation => "InvalidCalculation".into(),
            StellarSpendError::StorageError => "StorageError".into(),
            StellarSpendError::CorruptedData => "CorruptedData".into(),
            StellarSpendError::DataNotFound => "DataNotFound".into(),
            StellarSpendError::WriteFailed => "WriteFailed".into(),
            StellarSpendError::ReadFailed => "ReadFailed".into(),
            StellarSpendError::NetworkError => "NetworkError".into(),
            StellarSpendError::ExternalCallFailed => "ExternalCallFailed".into(),
            StellarSpendError::OracleUnavailable => "OracleUnavailable".into(),
            StellarSpendError::BridgeError => "BridgeError".into(),
            StellarSpendError::TransactionFailed => "TransactionFailed".into(),
            StellarSpendError::ConditionNotMet => "ConditionNotMet".into(),
            StellarSpendError::DeadlineExceeded => "DeadlineExceeded".into(),
            StellarSpendError::IncompatibleOperation => "IncompatibleOperation".into(),
            StellarSpendError::InvalidOperation => "InvalidOperation".into(),
            StellarSpendError::SecurityViolation => "SecurityViolation".into(),
            StellarSpendError::SuspiciousActivity => "SuspiciousActivity".into(),
            StellarSpendError::BlacklistedAddress => "BlacklistedAddress".into(),
            StellarSpendError::FrozenAccount => "FrozenAccount".into(),
            StellarSpendError::ComplianceViolation => "ComplianceViolation".into(),
            StellarSpendError::SystemError => "SystemError".into(),
            StellarSpendError::InternalError => "InternalError".into(),
            StellarSpendError::NotImplemented => "NotImplemented".into(),
            StellarSpendError::MaintenanceMode => "MaintenanceMode".into(),
            StellarSpendError::UpgradeRequired => "UpgradeRequired".into(),
        }
    }

    /// Get detailed error description
    fn error_description(error: &StellarSpendError) -> String {
        match error {
            StellarSpendError::NotInitialized => "Contract has not been initialized".into(),
            StellarSpendError::AlreadyInitialized => "Contract has already been initialized".into(),
            StellarSpendError::InvalidInitialization => {
                "Invalid initialization parameters provided".into()
            }
            StellarSpendError::Unauthorized => {
                "Caller is not authorized to perform this operation".into()
            }
            StellarSpendError::InvalidSignature => "Provided signature is invalid".into(),
            StellarSpendError::InsufficientPermissions => {
                "Insufficient permissions for this operation".into()
            }
            StellarSpendError::AdminRequired => {
                "Admin privileges required for this operation".into()
            }
            StellarSpendError::MinterRequired => {
                "Minter privileges required for this operation".into()
            }
            StellarSpendError::InvalidInput => "Invalid input provided".into(),
            StellarSpendError::InvalidAmount => "Invalid amount provided".into(),
            StellarSpendError::InvalidAddress => "Invalid address provided".into(),
            StellarSpendError::InvalidTimestamp => "Invalid timestamp provided".into(),
            StellarSpendError::InvalidParameter => "Invalid parameter provided".into(),
            StellarSpendError::InvalidConfiguration => "Invalid configuration provided".into(),
            StellarSpendError::InvalidTransaction => "Invalid transaction provided".into(),
            StellarSpendError::InvalidSignatureFormat => "Invalid signature format".into(),
            StellarSpendError::NotFound => "Requested resource not found".into(),
            StellarSpendError::AlreadyExists => "Resource already exists".into(),
            StellarSpendError::InvalidState => {
                "Contract is in invalid state for this operation".into()
            }
            StellarSpendError::NotActive => "Contract or resource is not active".into(),
            StellarSpendError::Expired => "Resource has expired".into(),
            StellarSpendError::Locked => "Resource is currently locked".into(),
            StellarSpendError::Paused => "Contract is currently paused".into(),
            StellarSpendError::InsufficientBalance => {
                "Insufficient balance for this operation".into()
            }
            StellarSpendError::InsufficientAllowance => {
                "Insufficient allowance for this operation".into()
            }
            StellarSpendError::InsufficientLiquidity => "Insufficient liquidity available".into(),
            StellarSpendError::AmountExceedsLimit => "Amount exceeds allowed limit".into(),
            StellarSpendError::NegativeAmount => "Negative amount provided".into(),
            StellarSpendError::ZeroAmount => "Zero amount provided".into(),
            StellarSpendError::AmountTooLarge => "Amount is too large".into(),
            StellarSpendError::AmountTooSmall => "Amount is too small".into(),
            StellarSpendError::LimitExceeded => "Operation limit exceeded".into(),
            StellarSpendError::CapExceeded => "Cap limit exceeded".into(),
            StellarSpendError::QuotaExceeded => "Quota limit exceeded".into(),
            StellarSpendError::RateLimitExceeded => "Rate limit exceeded".into(),
            StellarSpendError::MaxUsersExceeded => "Maximum users exceeded".into(),
            StellarSpendError::MaxTransactionsExceeded => "Maximum transactions exceeded".into(),
            StellarSpendError::Overflow => "Arithmetic overflow detected".into(),
            StellarSpendError::Underflow => "Arithmetic underflow detected".into(),
            StellarSpendError::DivisionByZero => "Division by zero attempted".into(),
            StellarSpendError::InvalidCalculation => "Invalid calculation performed".into(),
            StellarSpendError::StorageError => "Storage operation failed".into(),
            StellarSpendError::CorruptedData => "Data corruption detected".into(),
            StellarSpendError::DataNotFound => "Requested data not found in storage".into(),
            StellarSpendError::WriteFailed => "Failed to write to storage".into(),
            StellarSpendError::ReadFailed => "Failed to read from storage".into(),
            StellarSpendError::NetworkError => "Network operation failed".into(),
            StellarSpendError::ExternalCallFailed => "External contract call failed".into(),
            StellarSpendError::OracleUnavailable => "Oracle service is unavailable".into(),
            StellarSpendError::BridgeError => "Bridge operation failed".into(),
            StellarSpendError::TransactionFailed => "Transaction execution failed".into(),
            StellarSpendError::ConditionNotMet => "Required condition not met".into(),
            StellarSpendError::DeadlineExceeded => "Operation deadline exceeded".into(),
            StellarSpendError::IncompatibleOperation => "Incompatible operation attempted".into(),
            StellarSpendError::InvalidOperation => "Invalid operation attempted".into(),
            StellarSpendError::SecurityViolation => "Security violation detected".into(),
            StellarSpendError::SuspiciousActivity => "Suspicious activity detected".into(),
            StellarSpendError::BlacklistedAddress => "Address is blacklisted".into(),
            StellarSpendError::FrozenAccount => "Account is frozen".into(),
            StellarSpendError::ComplianceViolation => "Compliance rule violation".into(),
            StellarSpendError::SystemError => "System error occurred".into(),
            StellarSpendError::InternalError => "Internal error occurred".into(),
            StellarSpendError::NotImplemented => "Feature not implemented".into(),
            StellarSpendError::MaintenanceMode => "System is in maintenance mode".into(),
            StellarSpendError::UpgradeRequired => "Contract upgrade required".into(),
        }
    }

    /// Get common causes for this error
    fn error_causes(error: &StellarSpendError) -> Vec<String> {
        let env = &soroban_sdk::Env::default(); // This would be passed in real usage
        let mut causes = Vec::new(env);

        match error {
            StellarSpendError::NotInitialized => {
                causes.push_back("Contract initialization not completed".into());
                causes.push_back("Admin setup not performed".into());
            }
            StellarSpendError::Unauthorized => {
                causes.push_back("Caller lacks required permissions".into());
                causes.push_back("Invalid authentication provided".into());
            }
            StellarSpendError::InsufficientBalance => {
                causes.push_back("Account balance too low".into());
                causes.push_back("Recent transactions reduced balance".into());
            }
            StellarSpendError::RateLimitExceeded => {
                causes.push_back("Too many requests in time window".into());
                causes.push_back("Rate limit quota exceeded".into());
            }
            _ => {
                causes.push_back("Unknown specific cause".into());
            }
        }

        causes
    }

    /// Get suggested solutions for this error
    fn error_solutions(error: &StellarSpendError) -> Vec<String> {
        let env = &soroban_sdk::Env::default(); // This would be passed in real usage
        let mut solutions = Vec::new(env);

        match error {
            StellarSpendError::NotInitialized => {
                solutions.push_back("Initialize the contract first".into());
                solutions.push_back("Contact contract administrator".into());
            }
            StellarSpendError::Unauthorized => {
                solutions.push_back("Check your permissions".into());
                solutions.push_back("Use authorized account".into());
            }
            StellarSpendError::InsufficientBalance => {
                solutions.push_back("Add funds to your account".into());
                solutions.push_back("Reduce transaction amount".into());
            }
            StellarSpendError::RateLimitExceeded => {
                solutions.push_back("Wait before retrying".into());
                solutions.push_back("Reduce request frequency".into());
            }
            _ => {
                solutions.push_back("Contact support for assistance".into());
                solutions.push_back("Check error documentation".into());
            }
        }

        solutions
    }
}

/// Helper functions for error handling
pub struct ErrorHelpers;

impl ErrorHelpers {
    /// Create error context for logging
    pub fn create_context(
        env: &Env,
        error_code: u32,
        contract_name: &str,
        function_name: &str,
        parameters: Vec<String>,
        additional_info: Map<String, String>,
    ) -> ErrorContext {
        ErrorContext {
            error_code,
            contract_name: contract_name.into(),
            function_name: function_name.into(),
            parameters,
            timestamp: env.ledger().timestamp(),
            additional_info,
        }
    }

    /// Check if error should be logged
    pub fn should_log(error_code: u32) -> bool {
        match error_code {
            // Always log critical and high severity errors
            2000..=2199 => true, // System and Security
            1600..=1699 => true, // Arithmetic
            1700..=1799 => true, // Storage

            // Log medium severity errors selectively
            1100..=1199 => true, // Authorization
            1400..=1499 => true, // Balance/Amount

            // Don't log low severity informational errors
            _ => false,
        }
    }

    /// Get suggested retry strategy
    pub fn retry_strategy(error_code: u32) -> RetryStrategy {
        match error_code {
            // Immediate retry for transient errors
            1800 | 1802 => RetryStrategy::Immediate,

            // Exponential backoff for rate limits
            1503 => RetryStrategy::ExponentialBackoff,

            // Fixed delay for maintenance
            2103 => RetryStrategy::FixedDelay,

            // No retry for permanent errors
            1100 | 2000 | 1400 => RetryStrategy::NoRetry,

            // Default to exponential backoff
            _ => RetryStrategy::ExponentialBackoff,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum RetryStrategy {
    NoRetry = 0,
    Immediate = 1,
    FixedDelay = 2,
    ExponentialBackoff = 3,
}
