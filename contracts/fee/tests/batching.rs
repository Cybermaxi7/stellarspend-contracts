mod support;

use support::{amounts, setup};

#[test]
fn batch_collection_aggregates_fee_inputs_with_one_state_update() {
    let ctx = setup();
    let batch = amounts(&ctx.env, &[10, 20, 30, 40]);

    let result = ctx.client.collect_fee_batch(&ctx.payer, &batch);
    assert_eq!(result.batch_size, 4);
    assert_eq!(result.total_amount, 100);
    assert_eq!(result.cycle, 1);
    assert_eq!(result.pending_fees, 100);

    assert_eq!(ctx.client.get_pending_fees(&1), 100);
    assert_eq!(ctx.client.get_escrow_balance(), 100);
    assert_eq!(ctx.client.get_total_collected(), 100);
    assert_eq!(ctx.client.get_total_batch_calls(), 1);
    assert_eq!(ctx.token_client.balance(&ctx.contract_id), 100);
    assert_eq!(ctx.token_client.balance(&ctx.payer), 999_900);
}

#[test]
#[should_panic]
fn batch_collection_rejects_invalid_fee_amounts() {
    let ctx = setup();
    let batch = amounts(&ctx.env, &[10, 0, 30]);
    ctx.client.collect_fee_batch(&ctx.payer, &batch);
}
