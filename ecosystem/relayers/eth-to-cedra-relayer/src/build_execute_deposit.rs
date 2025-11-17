use aptos_sdk::{
    move_types::{
        identifier::Identifier,
        language_storage::ModuleId
    },
    types::{
        account_address::AccountAddress,
        transaction::{EntryFunction, TransactionPayload},
    },
    bcs,
};

/// Build the EntryFunction payload for bridge::execute_deposit (not wrapped in multisig).
pub fn build_execute_deposit_entry(
    bridge_module_address: AccountAddress,
    l1_token_20: Vec<u8>,   // exactly 20 bytes
    to_cedra: AccountAddress,
    amount: u64,
    nonce: u64,
    eth_tx_hash_32: Vec<u8>, // exactly 32 bytes
) -> EntryFunction {
    let module = ModuleId::new(
        bridge_module_address,
        Identifier::new("bridge").unwrap(),
    );

    let func = Identifier::new("execute_deposit").unwrap();

    EntryFunction::new(
        module,
        func,
        vec![],
        vec![
            bcs::to_bytes(&l1_token_20).unwrap(),
            bcs::to_bytes(&to_cedra).unwrap(),
            bcs::to_bytes(&amount).unwrap(),
            bcs::to_bytes(&nonce).unwrap(),
            bcs::to_bytes(&eth_tx_hash_32).unwrap(),
        ],
    )
}