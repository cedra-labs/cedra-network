module cedra_framework::bridge {
    use std::create_signer;
    use std::string;
    use std::error;
    use std::signer;
    use std::vector;

    use cedra_std::table::{Self as table, Table};
    use cedra_std::type_info;

    use cedra_framework::coin;
    use cedra_framework::event::{Self as ev, EventHandle};
    use cedra_framework::guid;

    #[event]
    struct BalanceLog has drop, store {
        label: string::String,
        addr: address,
        before: u64,
        after: u64,
    }

    /// ============= Errors =============
    const E_NOT_ADMIN: u64 = 1;
    const E_BAD_INPUT: u64 = 2;
    const E_VALIDATOR_EXISTS: u64 = 3;
    const E_VALIDATOR_UNKNOWN: u64 = 4;
    const E_ALREADY_ACTIVE: u64 = 5;
    const E_NOT_VALIDATOR: u64 = 6;
    const E_DUP_APPROVAL: u64 = 7;
    const E_REQUEST_EXECUTED: u64 = 8;
    const E_NONCE_USED: u64 = 9;
    const E_INSUFFICIENT_APPROVALS: u64 = 10;
    const E_PAUSED: u64 = 11;
    const E_ZERO_AMOUNT: u64 = 12;

    /// ============= Wrapped ETH coin type =============
    struct WETH has store, key {}

    /// Store mint/burn capabilities in a module resource.
    struct Caps has key {
        mint: coin::MintCapability<WETH>,
        burn: coin::BurnCapability<WETH>,
        freeze: coin::FreezeCapability<WETH>,
    }

    /// ============= Admin / pause =============
    struct Config has key {
        admin: address,
        paused: bool,
    }

    /// State change events (kept for possible future use).
    struct PausedEvent has drop, store { by: address }
    struct UnpausedEvent has drop, store { by: address }

    /// ============= Bridge validators =============
    struct ValidatorInfo has copy, drop, store {
        weight: u64,
        active: bool,
    }

    struct BridgeValidatorSet has key {
        validators: Table<address, ValidatorInfo>,
        list: vector<address>,
        total_weight: u64,
        updated: EventHandle<ValidatorSetUpdated>,
    }

    struct ValidatorSetUpdated has drop, store { total_weight: u64 }

    /// ============= Mint request queues =============
    struct MintRequest has store {
        to: address,
        amount: u64,
        approvals_weight: u64,
        executed: bool,
        approved: Table<address, bool>,
    }

    struct Requests has key {
        by_nonce: Table<u64, MintRequest>,
        used_nonce: Table<u64, bool>,
    }

    /// Bridge events
    struct DepositObserved has drop, store {
        eth_tx_hash: vector<u8>,
        to: address,
        amount: u64,
        nonce: u64,
    }
    struct MintExecuted has drop, store {
        to: address,
        amount: u64,
        nonce: u64,
    }
    struct BurnForExit has drop, store {
        from: address,
        eth_recipient: vector<u8>, // 20 bytes ETH address
        amount: u64,
        nonce: u64,
    }

    struct BridgeEvents has key {
        deposit_observed: EventHandle<DepositObserved>,
        mint_executed: EventHandle<MintExecuted>,
        burn_for_exit: EventHandle<BurnForExit>,
        balance_logs:    EventHandle<BalanceLog>,
    }

    /// ============= Module initialization =============
    public entry fun init(admin: &signer) {
        // publish config
        let cfg = Config { admin: signer::address_of(admin), paused: false };
        move_to(admin, cfg);

        let fw = create_signer::create_signer(@cedra_framework);
        coin::create_coin_conversion_map(&fw);

        // initialize WETH
        if (!coin::is_coin_initialized<WETH>()) {
            // the signer must match the type's account address
            let weth_addr = type_info::account_address(&type_info::type_of<WETH>());
            let coin_owner = create_signer::create_signer(weth_addr);

            let (burn, freeze, mint) = coin::initialize<WETH>(
                &coin_owner,
                string::utf8(b"Wrapped Ether"),
                string::utf8(b"WETH"),
                18,      // or your decimals
                true
            );

            // store the caps where your module expects them (e.g., under admin)
            move_to(admin, Caps { burn, freeze, mint });
        };

        // GUIDs: (address, creation_num)
        let admin_addr = signer::address_of(admin);
        let n1 = 1; let g1 = guid::create(admin_addr, &mut n1);
        let n2 = 2; let g2 = guid::create(admin_addr, &mut n2);
        let n3 = 3; let g3 = guid::create(admin_addr, &mut n3);
        let n4 = 4; let g4 = guid::create(admin_addr, &mut n4);

        // --- add one more GUID for balance log events ---
        let n5 = 5; 
        let g5 = guid::create(admin_addr, &mut n5);

        // empty validator set
        let vs = BridgeValidatorSet {
            validators: table::new<address, ValidatorInfo>(),
            list: vector::empty<address>(),
            total_weight: 0,
            updated: ev::new_event_handle<ValidatorSetUpdated>(g1),
        };
        move_to(admin, vs);

        // tables for requests and used nonces
        move_to(admin, Requests {
            by_nonce: table::new<u64, MintRequest>(),
            used_nonce: table::new<u64, bool>(),
        });

        // bridge events
        move_to(admin, BridgeEvents {
            deposit_observed: ev::new_event_handle<DepositObserved>(g2),
            mint_executed:   ev::new_event_handle<MintExecuted>(g3),
            burn_for_exit:   ev::new_event_handle<BurnForExit>(g4),
            balance_logs:    ev::new_event_handle<BalanceLog>(g5),
        });
    }

    /// ============= Admin and pause =============
    public entry fun pause(caller: &signer) acquires Config, BridgeEvents {
        let cfg = borrow_global_mut<Config>(signer::address_of(caller));
        assert!(is_admin(caller, cfg), error::permission_denied(E_NOT_ADMIN));
        cfg.paused = true;
        let evs = borrow_global_mut<BridgeEvents>(signer::address_of(caller));
        ev::emit_event(
            &mut evs.deposit_observed,
            DepositObserved { eth_tx_hash: vector::empty<u8>(), to: signer::address_of(caller), amount: 0, nonce: 0 }
        );
    }

    public entry fun unpause(caller: &signer) acquires Config {
        let cfg = borrow_global_mut<Config>(signer::address_of(caller));
        assert!(is_admin(caller, cfg), error::permission_denied(E_NOT_ADMIN));
        cfg.paused = false;
    }

    inline fun ensure_not_paused(admin_addr: address) acquires Config {
        let cfg = borrow_global<Config>(admin_addr);
        assert!(!cfg.paused, E_PAUSED);
    }

    inline fun is_admin(caller: &signer, cfg: &Config): bool {
        signer::address_of(caller) == cfg.admin
    }

    /// ============= Validator flow =============
    public entry fun initialize_validator(admin: &signer, validator: address, weight: u64)
    acquires Config, BridgeValidatorSet {
        assert!(weight > 0, E_BAD_INPUT);
        let cfg = borrow_global<Config>(signer::address_of(admin));
        assert!(is_admin(admin, cfg), error::permission_denied(E_NOT_ADMIN));

        let set = borrow_global_mut<BridgeValidatorSet>(signer::address_of(admin));
        let present = table::contains(&set.validators, validator);
        assert!(!present, E_VALIDATOR_EXISTS);

        table::add(&mut set.validators, validator, ValidatorInfo { weight, active: false });
        vector::push_back(&mut set.list, validator);
    }

    public entry fun join_validator_set(admin: &signer, validator: address)
    acquires Config, BridgeValidatorSet {
        let cfg = borrow_global<Config>(signer::address_of(admin));
        assert!(is_admin(admin, cfg), error::permission_denied(E_NOT_ADMIN));

        let set = borrow_global_mut<BridgeValidatorSet>(signer::address_of(admin));
        // TAKE the row out, mutate, put it back
        let old = table::remove(&mut set.validators, validator);
        assert!(!old.active, E_ALREADY_ACTIVE);
        let new = ValidatorInfo { weight: old.weight, active: true };
        set.total_weight = set.total_weight + new.weight;
        table::add(&mut set.validators, validator, new);

        ev::emit_event(&mut set.updated, ValidatorSetUpdated { total_weight: set.total_weight });
    }

    public entry fun leave_validator_set(admin: &signer, validator: address)
    acquires Config, BridgeValidatorSet {
        let cfg = borrow_global<Config>(signer::address_of(admin));
        assert!(is_admin(admin, cfg), error::permission_denied(E_NOT_ADMIN));

        let set = borrow_global_mut<BridgeValidatorSet>(signer::address_of(admin));
        // TAKE the row out, mutate, put it back
        let old = table::remove(&mut set.validators, validator);
        if (old.active) {
            let new = ValidatorInfo { weight: old.weight, active: false };
            set.total_weight = set.total_weight - old.weight;
            table::add(&mut set.validators, validator, new);
            ev::emit_event(&mut set.updated, ValidatorSetUpdated { total_weight: set.total_weight });
        } else {
            // put it back unchanged if it was already inactive
            table::add(&mut set.validators, validator, old);
        };
    }

    fun validator_weight(admin_addr: address, who: address): (bool, u64) acquires BridgeValidatorSet {
        let set = borrow_global<BridgeValidatorSet>(admin_addr);
        if (!table::contains(&set.validators, who)) return (false, 0);
        let info = table::borrow(&set.validators, who);
        if (!info.active) return (false, 0);
        (true, info.weight)
    }

    /// ============= Inbound (mint) =============
    public entry fun open_mint_request(
        admin: &signer,
        to: address,
        amount: u64,
        nonce: u64,
        eth_tx_hash: vector<u8>,
    ) acquires Config, Requests, BridgeEvents {
        assert!(amount > 0, E_ZERO_AMOUNT);

        let cfg = borrow_global<Config>(signer::address_of(admin));
        assert!(is_admin(admin, cfg), error::permission_denied(E_NOT_ADMIN));
        ensure_not_paused(cfg.admin);

        let rq = borrow_global_mut<Requests>(signer::address_of(admin));
        assert!(!table::contains(&rq.used_nonce, nonce), E_NONCE_USED);

        let mr = MintRequest { to, amount, approvals_weight: 0, executed: false, approved: table::new<address, bool>() };
        table::add(&mut rq.by_nonce, nonce, mr);
        table::add(&mut rq.used_nonce, nonce, true);

        let evs = borrow_global_mut<BridgeEvents>(signer::address_of(admin));
        ev::emit_event(&mut evs.deposit_observed, DepositObserved { eth_tx_hash, to, amount, nonce });
    }

    public entry fun approve_mint(
        validator_signer: &signer,
        admin: &signer,
        nonce: u64
    ) acquires Requests, BridgeValidatorSet, Config {
        let admin_addr = signer::address_of(admin);
        ensure_not_paused(admin_addr);

        let sender = signer::address_of(validator_signer);
        let (is_val, w) = validator_weight(admin_addr, sender);
        assert!(is_val, E_NOT_VALIDATOR);

        let rq = borrow_global_mut<Requests>(admin_addr);
        let mr = table::borrow_mut(&mut rq.by_nonce, nonce);
        assert!(!mr.executed, E_REQUEST_EXECUTED);

        let seen = table::contains(&mr.approved, sender);
        assert!(!seen, E_DUP_APPROVAL);

        table::add(&mut mr.approved, sender, true);
        mr.approvals_weight = mr.approvals_weight + w;
    }

    public entry fun execute_mint(
        _caller: &signer,
        admin: &signer,
        nonce: u64
    ) acquires Requests, BridgeValidatorSet, Caps, BridgeEvents, Config {
        let admin_addr = signer::address_of(admin);
        ensure_not_paused(admin_addr);

        let set = borrow_global<BridgeValidatorSet>(admin_addr);
        let rq  = borrow_global_mut<Requests>(admin_addr);
        let mr  = table::borrow_mut(&mut rq.by_nonce, nonce);
        assert!(!mr.executed, E_REQUEST_EXECUTED);

        assert!(mr.approvals_weight * 100 > set.total_weight * 66, E_INSUFFICIENT_APPROVALS);

        let Caps { mint, burn: _, freeze: _ } = borrow_global<Caps>(admin_addr);
        let minted = coin::mint<WETH>(mr.amount, mint);
        coin::deposit(mr.to, minted);

        mr.executed = true;

        let evs = borrow_global_mut<BridgeEvents>(admin_addr);
        ev::emit_event(&mut evs.mint_executed, MintExecuted { to: mr.to, amount: mr.amount, nonce });
    }

    /// ============= Outbound (burn) =============
    public entry fun burn_for_ethereum_exit(
        user: &signer,
        admin: &signer,
        eth_recipient: vector<u8>,
        amount: u64,
        exit_nonce: u64
    ) acquires Requests, Caps, BridgeEvents, Config {
        let admin_addr = signer::address_of(admin);
        ensure_not_paused(admin_addr);
        assert!(amount > 0, E_ZERO_AMOUNT);
        assert!(vector::length(&eth_recipient) == 20, E_BAD_INPUT);

        let rq = borrow_global_mut<Requests>(admin_addr);
        assert!(!table::contains(&rq.used_nonce, exit_nonce), E_NONCE_USED);
        table::add(&mut rq.used_nonce, exit_nonce, true);

        let c = coin::withdraw<WETH>(user, amount);
        let Caps { burn, mint: _, freeze: _ } = borrow_global<Caps>(admin_addr);
        coin::burn<WETH>(c, burn);

        let evs = borrow_global_mut<BridgeEvents>(admin_addr);
        ev::emit_event(&mut evs.burn_for_exit, BurnForExit {
            from: signer::address_of(user),
            eth_recipient,
            amount,
            nonce: exit_nonce,
        });
    }

    /// ============= Views =============
    public fun total_weight(admin_addr: address): u64 acquires BridgeValidatorSet {
        borrow_global<BridgeValidatorSet>(admin_addr).total_weight
    }

    public fun is_active_validator(admin_addr: address, who: address): bool acquires BridgeValidatorSet {
        let (ok, _) = validator_weight(admin_addr, who);
        ok
    }

    /// Users can register CoinStore<WETH> to receive minted coins.
    public entry fun register_weth_store(user: &signer, _admin_addr: address) {
        coin::register<WETH>(user);
    }

    public fun total_weight_for(admin: &signer): u64 acquires BridgeValidatorSet {
        total_weight(signer::address_of(admin))
    }

    public fun is_active_validator_for(admin: &signer, who: address): bool acquires BridgeValidatorSet {
        let set = borrow_global<BridgeValidatorSet>(signer::address_of(admin));
        if (!table::contains(&set.validators, who)) return false;
        let info = table::borrow(&set.validators, who);
        info.active
    }

    #[test_only]
    public fun log_balance(
        admin_addr: address,
        label_b: vector<u8>,
        addr: address,
        before: u64,
        after: u64
    ) acquires BridgeEvents {
        let evs = borrow_global_mut<BridgeEvents>(admin_addr);
        ev::emit_event(
            &mut evs.balance_logs,
            BalanceLog { label: string::utf8(label_b), addr, before, after }
        );
    }

    #[test_only]
    public fun log_validator_state(
        admin_addr: address,
        label_b: vector<u8>,
        who: address
    ) acquires BridgeValidatorSet, BridgeEvents {
        let set = borrow_global<BridgeValidatorSet>(admin_addr);
        let exists = table::contains(&set.validators, who);
        let (active, weight) = if (exists) {
            let info = table::borrow(&set.validators, who);
            (info.active, info.weight)
        } else {
            (false, 0)
        };
        let evs = borrow_global_mut<BridgeEvents>(admin_addr);
        ev::emit_event(
            &mut evs.balance_logs, // reuse an existing handle if you prefer not to add a new one
            BalanceLog { label: string::utf8(label_b), addr: who, before: weight, after: set.total_weight }
        );
    }

    #[view]
    public fun validator_debug(
        admin_addr: address,
        who: address
    ): (bool , bool , u64 , u64)
    acquires BridgeValidatorSet {
        let set = borrow_global<BridgeValidatorSet>(admin_addr);
        let exists = table::contains(&set.validators, who);
        if (!exists) return (false, false, 0, set.total_weight);
        let info = table::borrow(&set.validators, who);
        (true, info.active, info.weight, set.total_weight)
    }
}