// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{
    move_vm_ext::{CedraMoveResolver, SessionExt},
    system_module_names::{CALCULATE_FA_FEE_FROM_QUOTES, PRICE_STORAGE_MODULE},
};
use cedra_types::{
    move_utils::as_move_value::AsMoveValue,
    on_chain_config::{
        fa_feed_address_bytes, new_price_feed_id, oracle_module_id, OnChainConfig, OracleConfig,
        OraclePrice, OracleQuote, CEDRA_FEED_ADDRESS, CEDRA_FEED_SYMBOL, GET_PRICE_BY_FEED_ID,
    },
};
use cedra_vm_types::module_and_script_storage::module_storage::CedraModuleStorage;
use move_binary_format::errors::{Location, PartialVMError};
use move_core_types::{
    account_address::AccountAddress,
    language_storage::ModuleId,
    value::MoveValue,
    vm_status::{StatusCode, VMStatus},
};
use move_vm_runtime::module_traversal::TraversalContext;
use move_vm_types::gas::GasMeter;

fn vm_error(msg: impl Into<String>) -> VMStatus {
    PartialVMError::new(StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION)
        .with_message(msg.into())
        .finish(Location::Module(PRICE_STORAGE_MODULE.clone()))
        .into_vm_status()
}

fn fetch_oracle_quote(
    session: &mut SessionExt<impl CedraMoveResolver>,
    module_storage: &impl CedraModuleStorage,
    gas_meter: &mut impl GasMeter,
    traversal_context: &mut TraversalContext,
    oracle_module: &ModuleId,
    feed_id: Vec<u8>,
) -> Result<OracleQuote, VMStatus> {
    let output = session
        .execute_function_bypass_visibility(
            oracle_module,
            GET_PRICE_BY_FEED_ID,
            vec![],
            vec![MoveValue::vector_u8(feed_id)
                .simple_serialize()
                .expect("vector<u8> is serializable")],
            gas_meter,
            traversal_context,
            module_storage,
        )
        .map_err(|err| err.into_vm_status())?;
    let bytes = output
        .return_values
        .first()
        .map(|(bytes, _)| bytes.as_slice())
        .ok_or_else(|| vm_error("oracle get_price_by_feed_id returned no value"))?;
    let price: OraclePrice = bcs::from_bytes(bytes)
        .map_err(|e| vm_error(format!("failed to decode oracle Price: {e}")))?;
    Ok(OracleQuote::from(&price))
}

fn call_calculate_from_quotes(
    session: &mut SessionExt<impl CedraMoveResolver>,
    module_storage: &impl CedraModuleStorage,
    gas_meter: &mut impl GasMeter,
    traversal_context: &mut TraversalContext,
    gas_used: u64,
    storage_refund: u64,
    txn_gas_price: u64,
    fa: &OracleQuote,
    cedra: &OracleQuote,
) -> Result<u64, VMStatus> {
    let output = session
        .execute_function_bypass_visibility(
            &PRICE_STORAGE_MODULE,
            CALCULATE_FA_FEE_FROM_QUOTES,
            vec![],
            vec![
                MoveValue::U64(gas_used).simple_serialize().unwrap(),
                MoveValue::U64(storage_refund).simple_serialize().unwrap(),
                MoveValue::U64(txn_gas_price).simple_serialize().unwrap(),
                fa.as_move_value().simple_serialize().unwrap(),
                cedra.as_move_value().simple_serialize().unwrap(),
            ],
            gas_meter,
            traversal_context,
            module_storage,
        )
        .map_err(|err| err.into_vm_status())?;
    output
        .return_values
        .first()
        .and_then(|(bytes, _)| bcs::from_bytes::<u64>(bytes).ok())
        .ok_or_else(|| vm_error("FA fee calculation returned no u64 value"))
}

/// Fetch FA + Cedra oracle prices from the address in `OracleConfig`, then compute the FA fee.
pub fn compute_fa_fee_v2(
    session: &mut SessionExt<impl CedraMoveResolver>,
    module_storage: &impl CedraModuleStorage,
    gas_meter: &mut impl GasMeter,
    traversal_context: &mut TraversalContext,
    gas_used: u64,
    storage_refund: u64,
    txn_gas_price: u64,
    fa_addr: AccountAddress,
    symbol: &[u8],
) -> Result<u64, VMStatus> {
    let oracle_addr = OracleConfig::fetch_config(session.resolver)
        .map(|config| config.addr)
        .ok_or_else(|| vm_error("OracleConfig missing; set at genesis or via governance"))?;
    if oracle_addr == AccountAddress::ZERO {
        return Err(vm_error("OracleConfig address is zero"));
    }

    let oracle_module = oracle_module_id(oracle_addr);
    let fa_feed_id = new_price_feed_id(&fa_feed_address_bytes(fa_addr), symbol);
    let cedra_feed_id = new_price_feed_id(CEDRA_FEED_ADDRESS, CEDRA_FEED_SYMBOL);

    let fa_quote = fetch_oracle_quote(
        session,
        module_storage,
        gas_meter,
        traversal_context,
        &oracle_module,
        fa_feed_id,
    )?;
    let cedra_quote = fetch_oracle_quote(
        session,
        module_storage,
        gas_meter,
        traversal_context,
        &oracle_module,
        cedra_feed_id,
    )?;

    call_calculate_from_quotes(
        session,
        module_storage,
        gas_meter,
        traversal_context,
        gas_used,
        storage_refund,
        txn_gas_price,
        &fa_quote,
        &cedra_quote,
    )
}

pub fn try_compute_fa_fee_v2_view(
    session: &mut SessionExt<impl CedraMoveResolver>,
    module_storage: &impl CedraModuleStorage,
    gas_meter: &mut impl GasMeter,
    traversal_context: &mut TraversalContext,
    arguments: &[Vec<u8>],
) -> Result<u64, VMStatus> {
    if arguments.len() != 5 {
        return Err(vm_error("calculate_fa_fee_v2 expects 5 arguments"));
    }
    let gas_used: u64 = bcs::from_bytes(&arguments[0])
        .map_err(|e| vm_error(format!("invalid gas_used: {e}")))?;
    let storage_refund: u64 = bcs::from_bytes(&arguments[1])
        .map_err(|e| vm_error(format!("invalid storage_fee_refunded: {e}")))?;
    let txn_gas_price: u64 = bcs::from_bytes(&arguments[2])
        .map_err(|e| vm_error(format!("invalid txn_gas_price: {e}")))?;
    let fa_addr: AccountAddress = bcs::from_bytes(&arguments[3])
        .map_err(|e| vm_error(format!("invalid fa_address: {e}")))?;
    let symbol: Vec<u8> = bcs::from_bytes(&arguments[4])
        .map_err(|e| vm_error(format!("invalid symbol: {e}")))?;

    compute_fa_fee_v2(
        session,
        module_storage,
        gas_meter,
        traversal_context,
        gas_used,
        storage_refund,
        txn_gas_price,
        fa_addr,
        &symbol,
    )
}
