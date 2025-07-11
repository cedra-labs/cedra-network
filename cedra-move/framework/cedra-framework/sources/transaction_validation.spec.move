spec cedra_framework::transaction_validation {
    /// <high-level-req>
    /// No.: 1
    /// Requirement: The sender of a transaction should have sufficient coin balance to pay the transaction fee.
    /// Criticality: High
    /// Implementation: The prologue_common function asserts that the transaction sender has enough coin balance to be
    /// paid as the max_transaction_fee.
    /// Enforcement: Formally verified via [high-level-req-1](PrologueCommonAbortsIf). Moreover, the native transaction validation patterns have been manually audited.
    ///
    /// No.: 2
    /// Requirement: All secondary signer addresses are verified to be authentic through a validation process.
    /// Criticality: Critical
    /// Implementation: The function multi_agent_script_prologue ensures that each secondary signer address undergoes
    /// authentication validation, including verification of account existence and authentication key matching,
    /// confirming their authenticity.
    /// Enforcement: Formally verified via [high-level-req-2](multi_agent_script_prologue). Moreover, the native transaction validation patterns have been manually audited.
    ///
    /// No.: 3
    /// Requirement: After successful execution, base the transaction fee on the configuration set by the features library.
    /// Criticality: High
    /// Implementation: The epilogue function collects the transaction fee for either redistribution or burning based on
    /// the feature::collect_and_distribute_gas_fees result.
    /// Enforcement: Formally Verified via [high-level-req-3](epilogue). Moreover, the native transaction validation patterns have been manually audited.
    /// </high-level-req>
    ///
    spec module {
        pragma verify = true;
        pragma aborts_if_is_strict;
    }

    spec grant_gas_permission(
        master: &signer,
        permissioned: &signer,
        gas_amount: u64
    ) {
        pragma aborts_if_is_partial;
    }

    spec revoke_gas_permission(permissioned: &signer) {
        pragma aborts_if_is_partial;
    }

    /// Ensure caller is `cedra_framework`.
    /// Aborts if TransactionValidation already exists.
    spec initialize(
        cedra_framework: &signer,
        script_prologue_name: vector<u8>,
        module_prologue_name: vector<u8>,
        multi_agent_prologue_name: vector<u8>,
        user_epilogue_name: vector<u8>,
    ) {
        use std::signer;
        let addr = signer::address_of(cedra_framework);
        aborts_if !system_addresses::is_cedra_framework_address(addr);
        aborts_if exists<TransactionValidation>(addr);

        ensures exists<TransactionValidation>(addr);
    }

    /// Create a schema to reuse some code.
    /// Give some constraints that may abort according to the conditions.
    spec schema PrologueCommonAbortsIf {
        use cedra_framework::timestamp::{CurrentTimeMicroseconds};
        use cedra_framework::chain_id::{ChainId};
        sender: &signer;
        gas_payer: &signer;
        replay_protector: ReplayProtector;
        txn_authentication_key: Option<vector<u8>>;
        txn_gas_price: u64;
        txn_max_gas_units: u64;
        txn_expiration_time: u64;
        chain_id: u8;

        aborts_if !exists<CurrentTimeMicroseconds>(@cedra_framework);
        aborts_if !(timestamp::now_seconds() < txn_expiration_time);

        aborts_if !exists<ChainId>(@cedra_framework);
        aborts_if !(chain_id::get() == chain_id);
        let transaction_sender = signer::address_of(sender);
        let gas_payer_addr = signer::address_of(gas_payer);
    }

    spec prologue_common(
        sender: &signer,
        gas_payer: &signer,
        replay_protector: ReplayProtector,
        txn_authentication_key: Option<vector<u8>>,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        txn_expiration_time: u64,
        chain_id: u8,
        is_simulation: bool,
    ) {
        // TODO(fa_migration)
        pragma verify = false;
        include PrologueCommonAbortsIf;
    }

    spec check_for_replay_protection_orderless_txn(
        sender: address,
        nonce: u64,
        txn_expiration_time: u64,
    ) {
        pragma verify = false;
    }

    spec check_for_replay_protection_regular_txn(
        sender_address: address,
        gas_payer_address: address,
        txn_sequence_number: u64,
    ) {
        pragma verify = false;
    }

    spec script_prologue_extended(
        sender: signer,
        txn_sequence_number: u64,
        txn_public_key: vector<u8>,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        txn_expiration_time: u64,
        chain_id: u8,
        _script_hash: vector<u8>,
        is_simulation: bool,
    ) {
        // TODO(fa_migration)
        pragma verify = false;
        include PrologueCommonAbortsIf {
            gas_payer: sender,
            txn_authentication_key: option::spec_some(txn_public_key),
            replay_protector: ReplayProtector::SequenceNumber(txn_sequence_number),
        };
    }

    spec script_prologue(
        sender: signer,
        txn_sequence_number: u64,
        txn_public_key: vector<u8>,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        txn_expiration_time: u64,
        chain_id: u8,
        _script_hash: vector<u8>,
    ) {
        // TODO: temporary mockup
        pragma verify = false;
    }

    spec schema MultiAgentPrologueCommonAbortsIf {
        secondary_signer_addresses: vector<address>;
        secondary_signer_public_key_hashes: vector<Option<vector<u8>>>;
        is_simulation: bool;

        // Vectors to be `zipped with` should be of equal length.
        let num_secondary_signers = len(secondary_signer_addresses);
        aborts_if len(secondary_signer_public_key_hashes) != num_secondary_signers;

        // If any account does not exist, or public key hash does not match, abort.
        // property 2: All secondary signer addresses are verified to be authentic through a validation process.
        /// [high-level-req-2]
        aborts_if exists i in 0..num_secondary_signers:
            !account::spec_exists_at(secondary_signer_addresses[i]);
        aborts_if exists i in 0..num_secondary_signers:
            !can_skip(features::spec_simulation_enhancement_enabled(), is_simulation, secondary_signer_public_key_hashes[i]) &&
                option::spec_is_some(secondary_signer_public_key_hashes[i]) && option::spec_borrow(
                secondary_signer_public_key_hashes[i]
            ) !=
                    account::spec_get_authentication_key(secondary_signer_addresses[i]);
        // By the end, all secondary signers account should exist and public key hash should match.
        ensures forall i in 0..num_secondary_signers:
            account::spec_exists_at(secondary_signer_addresses[i]);
        ensures forall i in 0..num_secondary_signers:
            option::spec_is_none(secondary_signer_public_key_hashes[i]) || option::spec_borrow(
                secondary_signer_public_key_hashes[i]
            ) ==
                account::spec_get_authentication_key(secondary_signer_addresses[i])
                || can_skip(features::spec_simulation_enhancement_enabled(), is_simulation, secondary_signer_public_key_hashes[i]);
    }

    spec fun can_skip(feature_flag: bool, is_simulation: bool, auth_key: Option<vector<u8>>): bool {
        features::spec_simulation_enhancement_enabled() && is_simulation && option::spec_is_none(auth_key)
    }

    spec multi_agent_common_prologue(
        secondary_signer_addresses: vector<address>,
        secondary_signer_public_key_hashes: vector<Option<vector<u8>>>,
        is_simulation: bool,
    ) {
        pragma aborts_if_is_partial;
        // include MultiAgentPrologueCommonAbortsIf {
        //     secondary_signer_addresses,
        //     secondary_signer_public_key_hashes,
        //     is_simulation,
        // };
    }

    /// Aborts if length of public key hashed vector
    /// not equal the number of singers.
    spec multi_agent_script_prologue_extended(
        sender: signer,
        txn_sequence_number: u64,
        txn_sender_public_key: vector<u8>,
        secondary_signer_addresses: vector<address>,
        secondary_signer_public_key_hashes: vector<vector<u8>>,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        txn_expiration_time: u64,
        chain_id: u8,
        is_simulation: bool,
    ) {
        pragma verify_duration_estimate = 120;
        let gas_payer = sender;
        // TODO(fa_migration)
        pragma verify = false;
        // include PrologueCommonAbortsIf {
        //     gas_payer,
        //     txn_sequence_number,
        //     txn_authentication_key: txn_sender_public_key,
        // };
        // include MultiAgentPrologueCommonAbortsIf {
        //     secondary_signer_addresses,
        //     vector::map(secondary_signer_public_key_hashes, |x| option::spec_some(x)),
        //     is_simulation,
        // };
    }

    spec multi_agent_script_prologue(
        sender: signer,
        txn_sequence_number: u64,
        txn_sender_public_key: vector<u8>,
        secondary_signer_addresses: vector<address>,
        secondary_signer_public_key_hashes: vector<vector<u8>>,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        txn_expiration_time: u64,
        chain_id: u8,
    ) {
        // TODO: temporary mockup
        pragma verify = false;
    }

    spec fee_payer_script_prologue_extended(
        sender: signer,
        txn_sequence_number: u64,
        txn_sender_public_key: vector<u8>,
        secondary_signer_addresses: vector<address>,
        secondary_signer_public_key_hashes: vector<vector<u8>>,
        fee_payer_address: address,
        fee_payer_public_key_hash: vector<u8>,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        txn_expiration_time: u64,
        chain_id: u8,
        is_simulation: bool,
    ) {
        pragma aborts_if_is_partial;
        pragma verify_duration_estimate = 120;

        aborts_if !features::spec_is_enabled(features::FEE_PAYER_ENABLED);
        let gas_payer = create_signer::create_signer(fee_payer_address);
        include PrologueCommonAbortsIf {
            gas_payer,
            replay_protector: ReplayProtector::SequenceNumber(txn_sequence_number),
            txn_authentication_key: option::spec_some(txn_sender_public_key),
        };
        // include MultiAgentPrologueCommonAbortsIf {
        //     secondary_signer_addresses,
        //     secondary_signer_public_key_hashes,
        //     is_simulation,
        // };

        aborts_if !account::spec_exists_at(fee_payer_address);
        aborts_if !(fee_payer_public_key_hash == account::spec_get_authentication_key(fee_payer_address));
        aborts_if !features::spec_fee_payer_enabled();
    }

    spec fee_payer_script_prologue(
        sender: signer,
        txn_sequence_number: u64,
        txn_sender_public_key: vector<u8>,
        secondary_signer_addresses: vector<address>,
        secondary_signer_public_key_hashes: vector<vector<u8>>,
        fee_payer_address: address,
        fee_payer_public_key_hash: vector<u8>,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        txn_expiration_time: u64,
        chain_id: u8,
    ) {
        // TODO: temporary mockup
        pragma verify = false;
    }

    /// Abort according to the conditions.
    /// `CedraCoinCapabilities` and `CoinInfo` should exists.
    /// Skip transaction_fee::burn_fee verification.
    spec epilogue_extended(
        account: signer,
        storage_fee_refunded: u64,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        gas_units_remaining: u64,
        is_simulation: bool,
    ) {
        // TODO(fa_migration)
        pragma verify = false;
        include EpilogueGasPayerAbortsIf { gas_payer: signer::address_of(account) };
    }

    spec epilogue(
        account: signer,
        storage_fee_refunded: u64,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        gas_units_remaining: u64,
    ) {
        // TODO: temporary mockup
        pragma verify = false;
    }

    /// Abort according to the conditions.
    /// `CedraCoinCapabilities` and `CoinInfo` should exist.
    /// Skip transaction_fee::burn_fee verification.
    spec epilogue_gas_payer_extended(
        account: signer,
        gas_payer: address,
        storage_fee_refunded: u64,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        gas_units_remaining: u64,
        is_simulation: bool,
    ) {
        // TODO(fa_migration)
        pragma verify = false;
        include EpilogueGasPayerAbortsIf;
    }

    spec epilogue_gas_payer(
        account: signer,
        gas_payer: address,
        storage_fee_refunded: u64,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        gas_units_remaining: u64,
    ) {
        // TODO: temporary mockup
        pragma verify = false;
    }

    spec unified_prologue(
        sender: signer,
        txn_sender_public_key: Option<vector<u8>>,
        txn_sequence_number: u64,
        secondary_signer_addresses: vector<address>,
        secondary_signer_public_key_hashes: vector<Option<vector<u8>>>,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        txn_expiration_time: u64,
        chain_id: u8,
        is_simulation: bool,
    ) {
        // TODO: temporary mockup
        pragma verify = false;
    }

    spec unified_prologue_fee_payer(
        sender: signer,
        fee_payer: signer,
        txn_sender_public_key: Option<vector<u8>>,
        fee_payer_public_key_hash: Option<vector<u8>>,
        txn_sequence_number: u64,
        secondary_signer_addresses: vector<address>,
        secondary_signer_public_key_hashes: vector<Option<vector<u8>>>,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        txn_expiration_time: u64,
        chain_id: u8,
        is_simulation: bool,
    ) {
        // TODO: temporary mockup
        pragma verify = false;
    }

    spec unified_epilogue(
        account: signer,
        gas_payer: signer,
        storage_fee_refunded: u64,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        gas_units_remaining: u64,
        is_simulation: bool,
    ) {
        // TODO: temporary mockup
        pragma verify = false;
    }

    spec unified_prologue_v2(
        sender: signer,
        txn_sender_public_key: Option<vector<u8>>,
        replay_protector: ReplayProtector,
        secondary_signer_addresses: vector<address>,
        secondary_signer_public_key_hashes: vector<Option<vector<u8>>>,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        txn_expiration_time: u64,
        chain_id: u8,
        is_simulation: bool,
    ) {
        // TODO: temporary mockup
        pragma verify = false;
    }

    spec unified_prologue_fee_payer_v2(
        sender: signer,
        fee_payer: signer,
        txn_sender_public_key: Option<vector<u8>>,
        fee_payer_public_key_hash: Option<vector<u8>>,
        replay_protector: ReplayProtector,
        secondary_signer_addresses: vector<address>,
        secondary_signer_public_key_hashes: vector<Option<vector<u8>>>,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        txn_expiration_time: u64,
        chain_id: u8,
        is_simulation: bool,
    ) {
        // TODO: temporary mockup
        pragma verify = false;
    }

    spec unified_epilogue_v2(
        account: signer,
        gas_payer: signer,
        storage_fee_refunded: u64,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        gas_units_remaining: u64,
        is_simulation: bool,
        is_orderless_txn: bool,
    ) {
        // TODO: temporary mockup
        pragma verify = false;
    }


    spec schema EpilogueGasPayerAbortsIf {
        use std::option;
        use cedra_std::type_info;
        use cedra_framework::account::{Account};
        use cedra_framework::cedra_coin::{CedraCoin};
        use cedra_framework::coin;
        use cedra_framework::coin::{CoinStore, CoinInfo};
        use cedra_framework::optional_aggregator;
        use cedra_framework::transaction_fee::{CedraCoinCapabilities, CedraCoinMintCapability};

        account: signer;
        gas_payer: address;
        storage_fee_refunded: u64;
        txn_gas_price: u64;
        txn_max_gas_units: u64;
        gas_units_remaining: u64;

        // Check transaction invariants.
        aborts_if !(txn_max_gas_units >= gas_units_remaining);
        let gas_used = txn_max_gas_units - gas_units_remaining;
        aborts_if !(txn_gas_price * gas_used <= MAX_U64);
        let transaction_fee_amount = txn_gas_price * gas_used;

        // Check account invariants.
        let addr = signer::address_of(account);
        // TODO(fa_migration)
        // let pre_balance = global<coin::CoinStore<CedraCoin>>(gas_payer).coin.value;
        // let post balance = global<coin::CoinStore<CedraCoin>>(gas_payer).coin.value;
        let pre_account = global<account::Account>(addr);
        let post account = global<account::Account>(addr);

        aborts_if !exists<CoinStore<CedraCoin>>(gas_payer);
        aborts_if !exists<Account>(addr);
        aborts_if !(global<Account>(addr).sequence_number < MAX_U64);
        // aborts_if pre_balance < transaction_fee_amount;
        // ensures balance == pre_balance - transaction_fee_amount + storage_fee_refunded;
        ensures account.sequence_number == pre_account.sequence_number + 1;

        // Check burning.
        //   (Check the total supply aggregator when enabled.)
        let amount_to_burn = transaction_fee_amount - storage_fee_refunded;
        let apt_addr = type_info::type_of<CedraCoin>().account_address;
        let maybe_apt_supply = global<CoinInfo<CedraCoin>>(apt_addr).supply;
        let total_supply_enabled = option::spec_is_some(maybe_apt_supply);
        let apt_supply = option::spec_borrow(maybe_apt_supply);
        let apt_supply_value = optional_aggregator::optional_aggregator_value(apt_supply);
        let post post_maybe_apt_supply = global<CoinInfo<CedraCoin>>(apt_addr).supply;
        let post post_apt_supply = option::spec_borrow(post_maybe_apt_supply);
        let post post_apt_supply_value = optional_aggregator::optional_aggregator_value(post_apt_supply);

        aborts_if amount_to_burn > 0 && !exists<CedraCoinCapabilities>(@cedra_framework);
        aborts_if amount_to_burn > 0 && !exists<CoinInfo<CedraCoin>>(apt_addr);
        aborts_if amount_to_burn > 0 && total_supply_enabled && apt_supply_value < amount_to_burn;
        ensures total_supply_enabled ==> apt_supply_value - amount_to_burn == post_apt_supply_value;

        // Check minting.
        let amount_to_mint = storage_fee_refunded - transaction_fee_amount;
        let total_supply = coin::supply<CedraCoin>;
        let post post_total_supply = coin::supply<CedraCoin>;

        aborts_if amount_to_mint > 0 && !exists<CoinStore<CedraCoin>>(addr);
        aborts_if amount_to_mint > 0 && !exists<CedraCoinMintCapability>(@cedra_framework);
        aborts_if amount_to_mint > 0 && total_supply + amount_to_mint > MAX_U128;
        ensures amount_to_mint > 0 ==> post_total_supply == total_supply + amount_to_mint;

        let cedra_addr = type_info::type_of<CedraCoin>().account_address;
        aborts_if (amount_to_mint != 0) && !exists<coin::CoinInfo<CedraCoin>>(cedra_addr);
        include coin::CoinAddAbortsIf<CedraCoin> { amount: amount_to_mint };
    }
}
