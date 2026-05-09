mod htlc_sigall_spending_conditions_tests;
mod htlc_spending_conditions_tests;
mod locktime_spending_conditions_tests;
mod p2pk_sigall_spending_conditions_tests;
mod p2pk_spending_conditions_tests;
mod pending_async_melt_tests;

#[test]
fn custom_melt_extra_is_preserved_when_processor_returns_no_extra() {
    let request_extra = serde_json::json!({ "amount": 5_000 });

    let merged = super::merge_custom_melt_extra(&request_extra, None);

    assert_eq!(merged, Some(request_extra));
}

#[test]
fn custom_melt_extra_merges_processor_response_extra() {
    let request_extra = serde_json::json!({ "amount": 5_000 });
    let response_extra = serde_json::json!({
        "fee_options": [
            {
                "fee_reserve": 213,
                "estimated_blocks": 1
            }
        ],
        "outpoint": null
    });

    let merged = super::merge_custom_melt_extra(&request_extra, Some(response_extra));

    assert_eq!(
        merged,
        Some(serde_json::json!({
            "amount": 5_000,
            "fee_options": [
                {
                    "fee_reserve": 213,
                    "estimated_blocks": 1
                }
            ],
            "outpoint": null
        }))
    );
}
