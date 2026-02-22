# Integrating RBAC into Existing Contracts

This guide shows how to integrate the Access Control RBAC system into existing StellarSpend contracts.

## Option 1: Standalone Access Control Contract

Deploy the access control contract separately and reference it from your contracts.

### Step 1: Deploy Access Control Contract

```rust
// Deploy the access control contract
let access_control_id = env.register_contract(None, AccessControlContract);
let access_control = AccessControlContractClient::new(&env, &access_control_id);

// Initialize with admin
access_control.initialize(&admin);
```

### Step 2: Store Access Control Address in Your Contract

```rust
#[contracttype]
pub enum DataKey {
    AccessControl,
    // ... other keys
}

#[contractimpl]
impl YourContract {
    pub fn initialize(env: Env, admin: Address, access_control: Address) {
        env.storage().instance().set(&DataKey::AccessControl, &access_control);
        // ... rest of initialization
    }
}
```

### Step 3: Use Access Control in Your Functions

```rust
use access_control::{AccessControlContractClient, Role};

#[contractimpl]
impl YourContract {
    pub fn sensitive_operation(env: Env, caller: Address) {
        caller.require_auth();

        // Get access control contract
        let access_control_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::AccessControl)
            .expect("Access control not configured");

        let access_control = AccessControlContractClient::new(&env, &access_control_addr);

        // Check if caller has required role
        if !access_control.has_role(&caller, &Role::Operator) {
            panic_with_error!(&env, YourError::Unauthorized);
        }

        // Proceed with operation
        // ...
    }
}
```

## Option 2: Embedded RBAC Module

Copy the RBAC logic directly into your contract for tighter integration.

### Step 1: Add RBAC Module to Your Contract

Create `contracts/your-contract/src/access_control.rs`:

```rust
use soroban_sdk::{contracttype, panic_with_error, Address, Env, Map};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Role {
    Admin = 0,
    User = 1,
    Operator = 2,
    Auditor = 3,
}

pub fn grant_role(env: &Env, user: &Address, role: Role) {
    let mut roles: Map<Role, bool> = env
        .storage()
        .instance()
        .get(&DataKey::UserRoles(user.clone()))
        .unwrap_or(Map::new(env));

    roles.set(role.clone(), true);
    env.storage()
        .instance()
        .set(&DataKey::UserRoles(user.clone()), &roles);

    env.events()
        .publish(("access_control", "role_granted"), (user, role));
}

pub fn has_role(env: &Env, user: &Address, role: Role) -> bool {
    let roles: Map<Role, bool> = env
        .storage()
        .instance()
        .get(&DataKey::UserRoles(user.clone()))
        .unwrap_or(Map::new(env));

    roles.get(role).unwrap_or(false)
}

pub fn require_role(env: &Env, caller: &Address, role: Role) {
    if !has_role(env, caller, role) {
        panic_with_error!(env, YourError::Unauthorized);
    }
}
```

### Step 2: Use in Your Contract

```rust
mod access_control;

use access_control::{Role, grant_role, has_role, require_role};

#[contractimpl]
impl YourContract {
    pub fn initialize(env: Env, admin: Address) {
        // Grant admin role
        grant_role(&env, &admin, Role::Admin);
        // ...
    }

    pub fn sensitive_operation(env: Env, caller: Address) {
        caller.require_auth();
        require_role(&env, &caller, Role::Operator);

        // Proceed with operation
        // ...
    }
}
```

## Common Patterns

### Pattern 1: Admin-Only Functions

```rust
pub fn admin_function(env: Env, caller: Address) {
    caller.require_auth();

    let access_control = get_access_control(&env);
    if !access_control.has_role(&caller, &Role::Admin) {
        panic_with_error!(&env, Error::Unauthorized);
    }

    // Admin-only logic
}
```

### Pattern 2: Multi-Role Functions

```rust
pub fn operator_or_admin_function(env: Env, caller: Address) {
    caller.require_auth();

    let access_control = get_access_control(&env);
    let is_admin = access_control.has_role(&caller, &Role::Admin);
    let is_operator = access_control.has_role(&caller, &Role::Operator);

    if !is_admin && !is_operator {
        panic_with_error!(&env, Error::Unauthorized);
    }

    // Logic for operators and admins
}
```

### Pattern 3: Role-Based Logic

```rust
pub fn flexible_function(env: Env, caller: Address) {
    caller.require_auth();

    let access_control = get_access_control(&env);

    if access_control.has_role(&caller, &Role::Admin) {
        // Admin gets full access
        perform_admin_operation(&env);
    } else if access_control.has_role(&caller, &Role::Operator) {
        // Operator gets limited access
        perform_operator_operation(&env);
    } else if access_control.has_role(&caller, &Role::User) {
        // User gets basic access
        perform_user_operation(&env);
    } else {
        panic_with_error!(&env, Error::Unauthorized);
    }
}
```

## Example: Updating Batch Transfer Contract

Here's how to add RBAC to the existing batch-transfer contract:

```rust
// In lib.rs
use access_control::{AccessControlContractClient, Role};

#[contracttype]
pub enum DataKey {
    Admin,
    AccessControl,  // Add this
    TotalBatches,
    TotalTransfersProcessed,
    TotalVolumeTransferred,
}

#[contractimpl]
impl BatchTransferContract {
    pub fn initialize(env: Env, admin: Address, access_control: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Contract already initialized");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::AccessControl, &access_control);
        // ... rest of initialization
    }

    pub fn batch_transfer(
        env: Env,
        caller: Address,
        token: Address,
        transfers: Vec<TransferRequest>,
    ) -> BatchTransferResult {
        caller.require_auth();

        // Use RBAC instead of simple admin check
        let access_control_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::AccessControl)
            .expect("Access control not configured");

        let access_control = AccessControlContractClient::new(&env, &access_control_addr);

        // Require admin OR operator role
        let is_admin = access_control.has_role(&caller, &Role::Admin);
        let is_operator = access_control.has_role(&caller, &Role::Operator);

        if !is_admin && !is_operator {
            panic_with_error!(&env, BatchTransferError::Unauthorized);
        }

        // ... rest of function
    }
}
```

## Testing with RBAC

```rust
#[test]
fn test_rbac_integration() {
    let env = Env::default();
    env.mock_all_auths();

    // Deploy access control
    let access_control_id = env.register_contract(None, AccessControlContract);
    let access_control = AccessControlContractClient::new(&env, &access_control_id);

    let admin = Address::generate(&env);
    access_control.initialize(&admin);

    // Deploy your contract
    let contract_id = env.register_contract(None, YourContract);
    let client = YourContractClient::new(&env, &contract_id);
    client.initialize(&admin, &access_control_id);

    // Grant operator role
    let operator = Address::generate(&env);
    access_control.grant_role(&admin, &operator, &Role::Operator);

    // Test operator can call function
    client.sensitive_operation(&operator);

    // Test unauthorized user cannot call
    let unauthorized = Address::generate(&env);
    // This should panic
    // client.sensitive_operation(&unauthorized);
}
```

## Best Practices

1. **Initialize Access Control First**: Always deploy and initialize the access control contract before your main contracts.

2. **Store Access Control Address**: Store the access control contract address in your contract's storage during initialization.

3. **Check Roles Early**: Perform role checks at the beginning of functions, right after `require_auth()`.

4. **Use Appropriate Roles**:
   - `Admin`: For critical operations like upgrades, admin transfers
   - `Operator`: For operational tasks like batch processing
   - `User`: For standard user operations
   - `Auditor`: For read-only access to sensitive data

5. **Emit Events**: The access control contract automatically emits events for all role changes, providing an audit trail.

6. **Test Thoroughly**: Always test both authorized and unauthorized access scenarios.

7. **Document Permissions**: Clearly document which roles are required for each function in your contract.

## Migration Strategy

For existing contracts without RBAC:

1. Deploy the access control contract
2. Initialize it with the current admin
3. Update your contract to accept the access control address
4. Gradually migrate functions to use RBAC
5. Grant appropriate roles to existing users
6. Test thoroughly before deploying to production

## Security Considerations

1. **Admin Protection**: The access control contract prevents admins from revoking their own admin role.
2. **Atomic Operations**: Role changes are atomic and emit events.
3. **No Role Hierarchy**: Roles are independent; having one role doesn't grant another.
4. **Explicit Checks**: Always explicitly check for required roles; don't assume.
5. **Audit Trail**: All role changes are logged via events for compliance and auditing.
