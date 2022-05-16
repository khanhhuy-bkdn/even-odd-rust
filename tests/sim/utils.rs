near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    // update `contract.wasm` for your contract's name
    CONTRACT_WASM_BYTES => "target/wasm32-unknown-unknown/release/contract.wasm",

    // if you run `cargo build` without `--release` flag:
    CONTRACT_WASM_BYTES => "target/wasm32-unknown-unknown/debug/contract.wasm",
}
use near_sdk_sim::{deploy, init_simulator, to_yocto, STORAGE_AMOUNT};
use token::EvenOddContract;

const CONTRACT_ID: &str = "contract";


pub fn init() -> (UserAccount, ContractAccount<EvenOddContract>, UserAccount) {
    let root = init_simulator(None);

    let contract = deploy!(
        contract: EvenOddContract,
        contract_id: CONTRACT_ID,
        bytes: &CONTRACT_WASM_BYTES,
        signer_account: root
    );

    let alice = root.create_user(
        "alice".parse().unwrap(),
        to_yocto("100") // initial balance
    );

    (root, contract, alice)
}