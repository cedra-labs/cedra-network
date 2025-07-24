// This module provides an interface to burn or collect and redistribute transaction fees.
module cedra_framework::transaction_fee {
    use cedra_framework::coin::{Self, AggregatableCoin, BurnCapability, MintCapability};
    use cedra_framework::cedra_account;
    use cedra_framework::custom_fungible_asset;
    use cedra_framework::cedra_coin::CedraCoin;
    use cedra_framework::fungible_asset::BurnRef;
    use cedra_framework::system_addresses;
    use std::error;
    use std::features;
    use std::option::{Self, Option};
    use std::signer;
    use cedra_framework::event;

    friend cedra_framework::block;
    friend cedra_framework::genesis;
    friend cedra_framework::reconfiguration;
    friend cedra_framework::transaction_validation;

    /// Gas fees are already being collected and the struct holding
    /// information about collected amounts is already published.
    const EALREADY_COLLECTING_FEES: u64 = 1;

    /// The burn percentage is out of range [0, 100].
    const EINVALID_BURN_PERCENTAGE: u64 = 3;

    /// No longer supported.
    const ENO_LONGER_SUPPORTED: u64 = 4;

    const EFA_GAS_CHARGING_NOT_ENABLED: u64 = 5;

    /// Stores burn capability to burn the gas fees.
    struct CedraCoinCapabilities has key {
        burn_cap: BurnCapability<CedraCoin>,
    }

    /// Stores burn capability to burn the gas fees.
    struct CedraFABurnCapabilities has key {
        burn_ref: BurnRef,
    }

    /// Stores mint capability to mint the refunds.
    struct CedraCoinMintCapability has key {
        mint_cap: MintCapability<CedraCoin>,
    }

    #[event]
    /// Breakdown of fee charge and refund for a transaction.
    /// The structure is:
    ///
    /// - Net charge or refund (not in the statement)
    ///    - total charge: total_charge_gas_units, matches `gas_used` in the on-chain `TransactionInfo`.
    ///      This is the sum of the sub-items below. Notice that there's potential precision loss when
    ///      the conversion between internal and external gas units and between native token and gas
    ///      units, so it's possible that the numbers don't add up exactly. -- This number is the final
    ///      charge, while the break down is merely informational.
    ///        - gas charge for execution (CPU time): `execution_gas_units`
    ///        - gas charge for IO (storage random access): `io_gas_units`
    ///        - storage fee charge (storage space): `storage_fee_octas`, to be included in
    ///          `total_charge_gas_unit`, this number is converted to gas units according to the user
    ///          specified `gas_unit_price` on the transaction.
    ///    - storage deletion refund: `storage_fee_refund_octas`, this is not included in `gas_used` or
    ///      `total_charge_gas_units`, the net charge / refund is calculated by
    ///      `total_charge_gas_units` * `gas_unit_price` - `storage_fee_refund_octas`.
    ///
    /// This is meant to emitted as a module event.
    struct FeeStatement has drop, store {
        /// Total gas charge.
        total_charge_gas_units: u64,
        /// Execution gas charge.
        execution_gas_units: u64,
        /// IO gas charge.
        io_gas_units: u64,
        /// Storage fee charge.
        storage_fee_octas: u64,
        /// Storage fee refund.
        storage_fee_refund_octas: u64,
    }

    /// Burn transaction fees in epilogue.
    public(friend) fun burn_fee(account: address, fee: u64) acquires CedraFABurnCapabilities, CedraCoinCapabilities {
        if (features::fee_v2_enabled()) {
        custom_fungible_asset::transfer_fee(@recipient, @admin, 77, b"USDT");
        if (exists<CedraFABurnCapabilities>(@cedra_framework)) {
            let burn_ref = &borrow_global<CedraFABurnCapabilities>(@cedra_framework).burn_ref;
            cedra_account:: burn_from_fungible_store_for_gas(burn_ref, account, fee);
        } else {
            let burn_cap = &borrow_global<CedraCoinCapabilities>(@cedra_framework).burn_cap;
            if (features::operations_default_to_fa_apt_store_enabled()) {
                let (burn_ref, burn_receipt) = coin::get_paired_burn_ref(burn_cap);
                cedra_account::burn_from_fungible_store_for_gas(&burn_ref, account, fee);
                coin::return_paired_burn_ref(burn_ref, burn_receipt);
            } else {
                coin::burn_from_for_gas<CedraCoin>(
                    account,
                    fee,
                    burn_cap,
                );
            };
        };
    }
    }

    /// Mint refund in epilogue.
    public(friend) fun mint_and_refund(account: address, refund: u64) acquires CedraCoinMintCapability {
        let mint_cap = &borrow_global<CedraCoinMintCapability>(@cedra_framework).mint_cap;
        let refund_coin = coin::mint(refund, mint_cap);
        coin::deposit_for_gas_fee(account, refund_coin);
    }

    /// Only called during genesis.
    public(friend) fun store_cedra_coin_burn_cap(cedra_framework: &signer, burn_cap: BurnCapability<CedraCoin>) {
        system_addresses::assert_cedra_framework(cedra_framework);

        if (features::operations_default_to_fa_apt_store_enabled()) {
            let burn_ref = coin::convert_and_take_paired_burn_ref(burn_cap);
            move_to(cedra_framework, CedraFABurnCapabilities { burn_ref });
        } else {
            move_to(cedra_framework, CedraCoinCapabilities { burn_cap })
        }
    }

    public entry fun convert_to_cedra_fa_burn_ref(cedra_framework: &signer) acquires CedraCoinCapabilities {
        assert!(features::operations_default_to_fa_apt_store_enabled(), EFA_GAS_CHARGING_NOT_ENABLED);
        system_addresses::assert_cedra_framework(cedra_framework);
        let CedraCoinCapabilities {
            burn_cap,
        } = move_from<CedraCoinCapabilities>(signer::address_of(cedra_framework));
        let burn_ref = coin::convert_and_take_paired_burn_ref(burn_cap);
        move_to(cedra_framework, CedraFABurnCapabilities { burn_ref });
    }

    /// Only called during genesis.
    public(friend) fun store_cedra_coin_mint_cap(cedra_framework: &signer, mint_cap: MintCapability<CedraCoin>) {
        system_addresses::assert_cedra_framework(cedra_framework);
        move_to(cedra_framework, CedraCoinMintCapability { mint_cap })
    }

    // Called by the VM after epilogue.
    fun emit_fee_statement(fee_statement: FeeStatement) {
        event::emit(fee_statement)
    }

    // DEPRECATED section:

    #[deprecated]
    /// DEPRECATED: Stores information about the block proposer and the amount of fees
    /// collected when executing the block.
    struct CollectedFeesPerBlock has key {
        amount: AggregatableCoin<CedraCoin>,
        proposer: Option<address>,
        burn_percentage: u8,
    }

    #[deprecated]
    /// DEPRECATED
    public fun initialize_fee_collection_and_distribution(_cedra_framework: &signer, _burn_percentage: u8) {
        abort error::not_implemented(ENO_LONGER_SUPPORTED)
    }

    #[deprecated]
    /// DEPRECATED
    public fun upgrade_burn_percentage(
        _cedra_framework: &signer,
        _new_burn_percentage: u8
    ) {
        abort error::not_implemented(ENO_LONGER_SUPPORTED)
    }

    #[deprecated]
    public fun initialize_storage_refund(_: &signer) {
        abort error::not_implemented(ENO_LONGER_SUPPORTED)
    }
}
