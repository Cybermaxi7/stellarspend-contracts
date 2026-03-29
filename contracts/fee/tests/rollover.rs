mod support;

use support::setup;

#[test]
fn pending_fees_roll_over_between_cycles_without_duplication() {
    let ctx = setup();

    ctx.client.collect_fee(&ctx.payer, &100i128);
    assert_eq!(ctx.client.get_pending_fees(&1), 100);
    assert_eq!(ctx.client.get_escrow_balance(), 100);

    let rolled = ctx.client.rollover_fees(&ctx.admin, &2u64);
    assert_eq!(rolled, 100);
    assert_eq!(ctx.client.get_current_cycle(), 2);
    assert_eq!(ctx.client.get_pending_fees(&1), 0);
    assert_eq!(ctx.client.get_pending_fees(&2), 100);
    assert_eq!(ctx.client.get_escrow_balance(), 100);
    assert_eq!(ctx.token_client.balance(&ctx.contract_id), 100);

    ctx.client.collect_fee(&ctx.payer, &40i128);
    assert_eq!(ctx.client.get_pending_fees(&2), 140);
    assert_eq!(ctx.client.get_escrow_balance(), 140);

    let rolled_again = ctx.client.rollover_fees(&ctx.admin, &3u64);
    assert_eq!(rolled_again, 140);
    assert_eq!(ctx.client.get_current_cycle(), 3);
    assert_eq!(ctx.client.get_pending_fees(&2), 0);
    assert_eq!(ctx.client.get_pending_fees(&3), 140);
    assert_eq!(ctx.client.get_escrow_balance(), 140);

    ctx.client.release_fees(&ctx.admin, &3u64);
    assert_eq!(ctx.client.get_pending_fees(&3), 0);
    assert_eq!(ctx.client.get_escrow_balance(), 0);
    assert_eq!(ctx.client.get_total_collected(), 140);
    assert_eq!(ctx.client.get_total_released(), 140);
    assert_eq!(ctx.token_client.balance(&ctx.treasury), 140);
}

#[test]
#[should_panic]
fn rollover_requires_a_newer_cycle() {
    let ctx = setup();
    ctx.client.rollover_fees(&ctx.admin, &1u64);
}
