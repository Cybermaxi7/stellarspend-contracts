mod support;

use support::setup;

#[test]
fn validate_config_happy_path() {
    let ctx = setup();
    // Defaults: fee_bps = 250 set in setup, min_fee = 0
    assert!(ctx.client.validate_config(&250u32, &0i128));
}

#[test]
#[should_panic]
fn initialize_rejects_invalid_fee_bps() {
    let ctx = setup();
    // lock/unlock not required; call setter which uses helper
    ctx.client.set_fee_bps(&ctx.admin, &10_001u32);
}

#[test]
#[should_panic]
fn set_min_fee_rejects_negative() {
    let ctx = setup();
    ctx.client.set_min_fee(&ctx.admin, &-5i128);
}

#[test]
fn setters_accept_valid_values() {
    let ctx = setup();
    ctx.client.set_fee_bps(&ctx.admin, &500u32);
    ctx.client.set_min_fee(&ctx.admin, &100i128);
    assert_eq!(ctx.client.get_fee_bps(), 500);
    assert_eq!(ctx.client.get_min_fee(), 100);
}
