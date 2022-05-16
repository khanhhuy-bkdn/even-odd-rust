use near_sdk::serde_json::json;
use near_sdk_sim::DEFAULT_GAS;

use crate::utils::init;

#[test]
fn simulate_some_view_function() {
    let (root, contract, _alice) = init();

    let actual: String = root.view(
        contract.account_id(),
        "get_balance",
        &json!({})
            .to_string()
            .into_bytes(),
    ).unwrap_json();

    assert_eq!("expected".to_string(), actual);
}

#[test]
fn simulate_some_change_method() {
    let (root, contract, _alice) = init();

    let result = root.call(
        contract.account_id(),
        "change_something",
        json!({}).to_string().into_bytes(),
        DEFAULT_GAS,
        to_yocto("0.0123"), // deposit
    );

    assert!(result.is_ok());
}