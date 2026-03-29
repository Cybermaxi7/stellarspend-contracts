mod support;

use support::setup;

#[test]
fn admin_can_toggle_lock_and_unlock() {
    let ctx = setup();

    assert!(!ctx.client.is_locked());

    ctx.client.lock(&ctx.admin);
    assert!(ctx.client.is_locked());

    ctx.client.unlock(&ctx.admin);
    assert!(!ctx.client.is_locked());
}

#[test]
#[should_panic]
fn lock_prevents_fee_bps_changes() {
    let ctx = setup();
    ctx.client.lock(&ctx.admin);
    ctx.client.set_fee_bps(&ctx.admin, &300u32);
}

#[test]
fn config_updates_work_after_unlock() {
    let ctx = setup();

    ctx.client.lock(&ctx.admin);
    ctx.client.unlock(&ctx.admin);
    ctx.client.set_fee_bps(&ctx.admin, &300u32);
    ctx.client.set_treasury(&ctx.admin, &ctx.alt_treasury);

    assert_eq!(ctx.client.get_fee_bps(), 300);
    assert_eq!(ctx.client.get_treasury(), ctx.alt_treasury);
}
