/// TODO: add tests & recheck batch_transfer logic
module cedra_framework::custom_account {
    use cedra_framework::account::{Self, new_event_handle};
    use cedra_framework::fungible_asset::{Self, Metadata, FungibleAsset};
    use cedra_framework::primary_fungible_store;
    use cedra_framework::object::{Self, Object};
    use cedra_framework::event::{EventHandle, emit_event, emit};
    
    use std::signer;
    use std::error;
    use std::vector;
        
    /// Account does not exist.
    const EACCOUNT_NOT_FOUND: u64 = 1;
    /// Account is not registered to receive fungible asset.
    const EACCOUNT_NOT_REGISTERED_FOR_FA: u64 = 2;
    /// Account opted out of receiving coins that they did not register to receive.
    const EACCOUNT_DOES_NOT_ACCEPT_DIRECT_COIN_TRANSFERS: u64 = 3;
    /// Account opted out of directly receiving NFT tokens.
    const EACCOUNT_DOES_NOT_ACCEPT_DIRECT_TOKEN_TRANSFERS: u64 = 4;
    /// The lengths of the recipients and amounts lists don't match.
    const EMISMATCHING_RECIPIENTS_AND_AMOUNTS_LENGTH: u64 = 5;

    /// Configuration for whether an account can receive direct transfers of coins that they have not registered.
    ///
    /// By default, this is enabled. Users can opt-out by disabling at any time.
    struct DirectTransferConfig has key {
        allow_arbitrary_coin_transfers: bool,
        update_coin_transfer_events: EventHandle<DirectCoinTransferConfigUpdatedEvent>,
    }

    /// Event emitted when an account's direct coins transfer config is updated.
    struct DirectCoinTransferConfigUpdatedEvent has drop, store {
        new_allow_direct_transfers: bool,
    }

    #[event]
    struct DirectCoinTransferConfigUpdated has drop, store {
        account: address,
        new_allow_direct_transfers: bool,
    }

    ///////////////////////////////////////////////////////////////////////////
    /// Basic account creation methods.
    ///////////////////////////////////////////////////////////////////////////

    public entry fun create_account(auth_key: address, fa_address: address) {
        let account_signer = account::create_account(auth_key);
        ensure_primary_fungible_store_exists(signer::address_of(&account_signer), fa_address);
    }

    /// Convenient function to transfer FA to a recipient account that might not exist.
    /// This would create the recipient account first, which also registers it to receive FA, before transferring.
    public entry fun transfer(from: address, to: address, amount: u64, fa_address: address) {
        if (!account::exists_at(to)) {
            create_account(to, fa_address)
        };

        fungible_transfer_only(from, to, amount, fa_address)
    }

       public entry fun batch_transfer_fungible_assets(
        from: &signer,
        metadata: Object<Metadata>,
        recipients: vector<address>,
        amounts: vector<u64>,
        fa_address: address
    ) {
        let recipients_len = vector::length(&recipients);
        assert!(
            recipients_len == vector::length(&amounts),
            error::invalid_argument(EMISMATCHING_RECIPIENTS_AND_AMOUNTS_LENGTH),
        );

        vector::enumerate_ref(&recipients, |i, to| {
            let amount = *vector::borrow(&amounts, i);
            transfer_fungible_assets(from, metadata, *to, amount, fa_address);
        });
    }

        /// Convenient function to deposit fungible asset into a recipient account that might not exist.
    /// This would create the recipient account first to receive the fungible assets.
    public entry fun transfer_fungible_assets(from: &signer, metadata: Object<Metadata>, to: address, amount: u64, fa_address: address) {
        deposit_fungible_assets(to, primary_fungible_store::withdraw(from, metadata, amount), fa_address);
    }


    /// Convenient function to deposit fungible asset into a recipient account that might not exist.
    /// This would create the recipient account first to receive the fungible assets.
    public fun deposit_fungible_assets(to: address, fa: FungibleAsset, fa_address: address) {
        if (!account::exists_at(to)) {
            create_account(to, fa_address);
        };
        primary_fungible_store::deposit(to, fa)
    }

    public fun assert_account_exists(addr: address) {
        assert!(account::exists_at(addr), error::not_found(EACCOUNT_NOT_FOUND));
    }

    public fun assert_account_is_registered_for_fa(addr: address) {
        assert_account_exists(addr);
        assert!(fungible_asset::store_exists(addr), error::not_found(EACCOUNT_NOT_REGISTERED_FOR_FA));
    }

     /// Set whether `account` can receive direct transfers of coins that they have not explicitly registered to receive.
    public entry fun set_allow_direct_coin_transfers(account: &signer, allow: bool) acquires DirectTransferConfig {
        let addr = signer::address_of(account);
        if (exists<DirectTransferConfig>(addr)) {
            let direct_transfer_config = borrow_global_mut<DirectTransferConfig>(addr);
            // Short-circuit to avoid emitting an event if direct transfer config is not changing.
            if (direct_transfer_config.allow_arbitrary_coin_transfers == allow) {
                return
            };

            direct_transfer_config.allow_arbitrary_coin_transfers = allow;

            if (std::features::module_event_migration_enabled()) {
                emit(DirectCoinTransferConfigUpdated { account: addr, new_allow_direct_transfers: allow });
            } else {
                emit_event(
                    &mut direct_transfer_config.update_coin_transfer_events,
                    DirectCoinTransferConfigUpdatedEvent { new_allow_direct_transfers: allow });
            };
        } else {
            let direct_transfer_config = DirectTransferConfig {
                allow_arbitrary_coin_transfers: allow,
                update_coin_transfer_events: new_event_handle<DirectCoinTransferConfigUpdatedEvent>(account),
            };
            if (std::features::module_event_migration_enabled()) {
                emit(DirectCoinTransferConfigUpdated { account: addr, new_allow_direct_transfers: allow });
            } else {
                emit_event(
                    &mut direct_transfer_config.update_coin_transfer_events,
                    DirectCoinTransferConfigUpdatedEvent { new_allow_direct_transfers: allow });
            };
            move_to(account, direct_transfer_config);
        };
    }

    #[view]
    /// Return true if `account` can receive direct transfers of coins that they have not explicitly registered to
    /// receive.
    ///
    /// By default, this returns true if an account has not explicitly set whether the can receive direct transfers.
    public fun can_receive_direct_coin_transfers(account: address): bool acquires DirectTransferConfig {
        !exists<DirectTransferConfig>(account) ||
            borrow_global<DirectTransferConfig>(account).allow_arbitrary_coin_transfers
    }

    /// Cedra Primary Fungible Store specific specialized functions,
    /// Utilized internally once migration of Cedra to FungibleAsset is complete.

    /// Convenient function to transfer Cedra to a recipient account that might not exist.
    /// This would create the recipient Cedra PFS first, which also registers it to receive Cedra, before transferring.
    /// TODO: once migration is complete, rename to just "transfer_only" and make it an entry function (for cheapest way
    /// to transfer Cedra) - if we want to allow Cedra PFS without account itself
    public(friend) entry fun fungible_transfer_only(
        from: address, to: address, amount: u64, fa_address: address
    ) {
        let sender_store = ensure_primary_fungible_store_exists(from , fa_address);
        let recipient_store = ensure_primary_fungible_store_exists(to, fa_address);

        // use internal APIs, as they skip:
        // - owner, frozen and dispatchable checks
        // as Cedra cannot be frozen or have dispatch, and PFS cannot be transfered
        // (PFS could potentially be burned. regular transfer would permanently unburn the store.
        // Ignoring the check here has the equivalent of unburning, transfers, and then burning again)
        // fungible_asset::withdraw_permission_check_by_address(from, sender_store, amount);
        fungible_asset::unchecked_deposit(recipient_store, fungible_asset::unchecked_withdraw(sender_store, amount));
    }

    /// Is balance from Cedra Primary FungibleStore at least the given amount
    public fun is_fungible_balance_at_least(account: address, amount: u64, fa_address: address): bool {
        let store_addr = object::create_user_derived_object_address(account, fa_address);
        fungible_asset::is_address_balance_at_least(store_addr, amount)
    }


    /// Ensure that Primary FungibleStore exists (and create if it doesn't)
    inline fun ensure_primary_fungible_store_exists(owner: address, fa_address: address): address {
        let store_addr = object::create_user_derived_object_address(owner, fa_address);
        if (fungible_asset::store_exists(store_addr)) {
            store_addr
        } else {
            object::object_address(&primary_fungible_store::create_primary_store(owner, object::address_to_object<Metadata>(fa_address)))
        }
    }
}
