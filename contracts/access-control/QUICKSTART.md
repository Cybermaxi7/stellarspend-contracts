# Quick Start Guide

Get up and running with RBAC in 5 minutes.

## 1. Add to Workspace

Already done! The access-control contract is in `contracts/access-control/`.

## 2. Build the Contract

```bash
cargo build -p access-control --target wasm32-unknown-unknown --release
```

## 3. Deploy (Example using Stellar CLI)

```bash
# Deploy access control contract
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/access_control.wasm \
  --source ADMIN_SECRET_KEY \
  --network testnet

# Save the contract ID
export ACCESS_CONTROL_ID="<contract-id>"
```

## 4. Initialize

```bash
# Initialize with admin address
stellar contract invoke \
  --id $ACCESS_CONTROL_ID \
  --source ADMIN_SECRET_KEY \
  --network testnet \
  -- initialize \
  --admin ADMIN_ADDRESS
```

## 5. Grant Roles

```bash
# Grant User role
stellar contract invoke \
  --id $ACCESS_CONTROL_ID \
  --source ADMIN_SECRET_KEY \
  --network testnet \
  -- grant_role \
  --caller ADMIN_ADDRESS \
  --user USER_ADDRESS \
  --role '{"User":{}}'

# Grant Operator role
stellar contract invoke \
  --id $ACCESS_CONTROL_ID \
  --source ADMIN_SECRET_KEY \
  --network testnet \
  -- grant_role \
  --caller ADMIN_ADDRESS \
  --user OPERATOR_ADDRESS \
  --role '{"Operator":{}}'
```

## 6. Check Roles

```bash
# Check if user has role
stellar contract invoke \
  --id $ACCESS_CONTROL_ID \
  --network testnet \
  -- has_role \
  --user USER_ADDRESS \
  --role '{"User":{}}'
```

## 7. Integrate into Your Contract

### Option A: Reference External Contract

```rust
use soroban_sdk::{contract, contractimpl, Address, Env};
use access_control::{AccessControlContractClient, Role};

#[contract]
pub struct MyContract;

#[contractimpl]
impl MyContract {
    pub fn initialize(env: Env, access_control: Address) {
        env.storage().instance().set(&DataKey::AccessControl, &access_control);
    }

    pub fn protected_function(env: Env, caller: Address) {
        caller.require_auth();

        let ac_addr: Address = env.storage().instance()
            .get(&DataKey::AccessControl).unwrap();
        let ac = AccessControlContractClient::new(&env, &ac_addr);

        if !ac.has_role(&caller, &Role::Operator) {
            panic!("Unauthorized");
        }

        // Your logic here
    }
}
```

### Option B: Copy Module Directly

Copy the role management functions into your contract's module.

## Common Commands

### Grant Role

```bash
stellar contract invoke --id $AC_ID -- grant_role \
  --caller ADMIN --user USER --role '{"User":{}}'
```

### Revoke Role

```bash
stellar contract invoke --id $AC_ID -- revoke_role \
  --caller ADMIN --user USER --role '{"User":{}}'
```

### Check Role

```bash
stellar contract invoke --id $AC_ID -- has_role \
  --user USER --role '{"User":{}}'
```

### Get All User Roles

```bash
stellar contract invoke --id $AC_ID -- get_user_roles \
  --user USER
```

### Transfer Admin

```bash
stellar contract invoke --id $AC_ID -- transfer_admin \
  --current_admin CURRENT_ADMIN --new_admin NEW_ADMIN
```

### Get Admin

```bash
stellar contract invoke --id $AC_ID -- get_admin
```

## Role Types

Use these exact formats when calling functions:

- Admin: `'{"Admin":{}}'`
- User: `'{"User":{}}'`
- Operator: `'{"Operator":{}}'`
- Auditor: `'{"Auditor":{}}'`

## Testing Locally

```bash
# Run all tests
cargo test -p access-control

# Run specific test
cargo test -p access-control test_grant_role_as_admin

# Run with output
cargo test -p access-control -- --nocapture
```

## Integration Checklist

- [ ] Deploy access control contract
- [ ] Initialize with admin
- [ ] Grant roles to users
- [ ] Update your contract to reference access control
- [ ] Add role checks to sensitive functions
- [ ] Test with different roles
- [ ] Deploy to testnet
- [ ] Verify permissions work correctly
- [ ] Deploy to mainnet

## Troubleshooting

### "Contract not initialized"

- Make sure you called `initialize()` after deployment

### "Unauthorized"

- Check that the caller has the required role
- Verify the admin address is correct
- Ensure `require_auth()` is called

### "Role already assigned"

- The user already has this role
- Use `has_role()` to check before granting

### "Cannot revoke self admin"

- Admins cannot revoke their own admin role
- Transfer admin first, then revoke

## Next Steps

1. Read [README.md](./README.md) for full API documentation
2. Check [INTEGRATION.md](./INTEGRATION.md) for integration patterns
3. Review [EXAMPLE.md](./EXAMPLE.md) for complete working example
4. See [SUMMARY.md](./SUMMARY.md) for implementation overview

## Support

For issues or questions:

1. Check the documentation files
2. Review the test suite in `src/test.rs`
3. Look at existing contracts for examples
4. Open an issue on GitHub

## Quick Reference

| Function         | Admin | Operator | User | Auditor |
| ---------------- | ----- | -------- | ---- | ------- |
| `initialize`     | ✅    | ❌       | ❌   | ❌      |
| `grant_role`     | ✅    | ❌       | ❌   | ❌      |
| `revoke_role`    | ✅    | ❌       | ❌   | ❌      |
| `transfer_admin` | ✅    | ❌       | ❌   | ❌      |
| `has_role`       | ✅    | ✅       | ✅   | ✅      |
| `get_user_roles` | ✅    | ✅       | ✅   | ✅      |
| `get_admin`      | ✅    | ✅       | ✅   | ✅      |

## Example Workflow

```bash
# 1. Deploy
stellar contract deploy --wasm access_control.wasm

# 2. Initialize
stellar contract invoke --id $ID -- initialize --admin $ADMIN

# 3. Grant roles
stellar contract invoke --id $ID -- grant_role \
  --caller $ADMIN --user $USER1 --role '{"User":{}}'

stellar contract invoke --id $ID -- grant_role \
  --caller $ADMIN --user $OPERATOR1 --role '{"Operator":{}}'

# 4. Verify
stellar contract invoke --id $ID -- has_role \
  --user $USER1 --role '{"User":{}}'
# Returns: true

# 5. Use in your contracts
# Reference $ID in your contract initialization
```

That's it! You now have a working RBAC system.
