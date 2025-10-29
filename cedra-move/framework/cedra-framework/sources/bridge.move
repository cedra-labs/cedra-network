module cedra_framework::bridge {
    use std::features;
    use std::signer;
    use std::string;
    use std::vector;

    use cedra_std::table::{Self as table, Table};

    use cedra_framework::account;
    use cedra_framework::coin;
    use cedra_framework::event::{Self as ev, EventHandle};
    use cedra_framework::system_addresses;

    /* ===================== Admin (multisig) ===================== */
    struct Admin has key {
        multisig: address,
    }

    /* ===================== Errors ===================== */
    const E_NOT_ADMIN: u64 = 1;
    const E_BAD_INPUT: u64 = 2;
    const E_ALREADY_ACTIVE: u64 = 5;
    const E_NONCE_USED: u64 = 9;
    const E_PAUSED: u64 = 11;
    const E_ZERO_AMOUNT: u64 = 12;
    const E_ALREADY_INITIALIZED: u64 = 13;
    const E_NO_APPROVAL: u64 = 14;
    const E_APPROVAL_MISMATCH: u64 = 15;

    /* ===================== Coin ===================== */
    struct WETH has store, key {}

    struct Caps has key {
        mint: coin::MintCapability<WETH>,
        burn: coin::BurnCapability<WETH>,
        freeze: coin::FreezeCapability<WETH>,
    }

    struct Config has key { paused: bool }

    /* ===================== Requests / Approvals ===================== */
    struct WithdrawalApproval has copy, drop, store {
        user: address,
        eth_recipient: vector<u8>, // 20 bytes
        amount: u64,
    }

    struct Requests has key {
        used_nonce: Table<u64, bool>,
        approvals: Table<u64, WithdrawalApproval>,
    }

    /* ===================== Events ===================== */
    #[event]
    struct DepositObserved has drop, store {
        eth_tx_hash: vector<u8>,
        to: address,
        amount: u64,
        nonce: u64,
    }

    #[event]
    struct MintExecuted has drop, store {
        to: address,
        amount: u64,
        nonce: u64,
    }

    #[event]
    struct Withdrawal has drop, store {
        from: address,
        eth_recipient: vector<u8>, // 20 bytes
        amount: u64,
        nonce: u64,
    }

    struct BridgeEvents has key {
        deposit_observed: EventHandle<DepositObserved>,
        mint_executed: EventHandle<MintExecuted>,
        withdrawal: EventHandle<Withdrawal>,
    }

    /* ===================== Helpers ===================== */
    inline fun assert_not_paused() acquires Config {
        assert!(!borrow_global<Config>(@cedra_framework).paused, E_PAUSED);
    }

    inline fun assert_framework(s: &signer) {
        system_addresses::assert_cedra_framework(s);
    }

    inline fun assert_multisig(s: &signer) acquires Admin {
        let admin = borrow_global<Admin>(@cedra_framework);
        assert!(signer::address_of(s) == admin.multisig, E_NOT_ADMIN);
    }

    /* ===================== Init & admin rotation ===================== */

    /// Call at genesis (or once after publish via framework account).
    public fun initialize(cedra_framework: &signer) {
        assert_framework(cedra_framework);
        assert!(!exists<Config>(@cedra_framework), E_ALREADY_INITIALIZED);

        move_to(cedra_framework, Config { paused: false });

        coin::create_coin_conversion_map(cedra_framework);
        if (!coin::is_coin_initialized<WETH>()) {
            let (burn, freeze, mint) = coin::initialize<WETH>(
                cedra_framework,
                string::utf8(b"Wrapped Ether"),
                string::utf8(b"WETH"),
                18,
                true
            );
            move_to(cedra_framework, Caps { burn, freeze, mint });
        };

        move_to(cedra_framework, Requests {
            used_nonce: table::new<u64, bool>(),
            approvals: table::new<u64, WithdrawalApproval>(),
        });

        move_to(cedra_framework, BridgeEvents {
            deposit_observed: account::new_event_handle<DepositObserved>(cedra_framework),
            mint_executed:   account::new_event_handle<MintExecuted>(cedra_framework),
            withdrawal:      account::new_event_handle<Withdrawal>(cedra_framework),
        });

        // Default multisig to the framework; rotate to the real multisig later.
        move_to(cedra_framework, Admin { multisig: signer::address_of(cedra_framework) });
    }

    /// Framework bootstrap: set the multisig address the first time (or any time, but requires framework signer).
    public entry fun set_multisig_framework_only(cedra_framework: &signer, addr: address) acquires Admin {
        assert_framework(cedra_framework);
        let a = borrow_global_mut<Admin>(@cedra_framework);
        a.multisig = addr;
    }

    /// Normal rotation by current multisig (k-of-n enforced in the multisig module).
    public entry fun rotate_multisig(multisig: &signer, new_addr: address) acquires Admin {
        assert_multisig(multisig);
        let a = borrow_global_mut<Admin>(@cedra_framework);
        a.multisig = new_addr;
    }

    public fun pause(multisig: &signer) acquires Config, Admin {
        assert_multisig(multisig);
        let cfg = borrow_global_mut<Config>(@cedra_framework);
        cfg.paused = true;
    }

    public fun unpause(multisig: &signer) acquires Config, Admin {
        assert_multisig(multisig);
        let cfg = borrow_global_mut<Config>(@cedra_framework);
        cfg.paused = false;
    }

    /* ===================== Deposit / Mint (multisig-gated) ===================== */

    /// Executed by multisig after off-chain/on-chain approvals.
    public entry fun execute_deposit(
        multisig: &signer,
        to: address,
        amount: u64,
        nonce: u64,
        eth_tx_hash: vector<u8>,
    ) acquires Requests, BridgeEvents, Caps, Config, Admin {
        assert_not_paused();
        assert_multisig(multisig);
        assert!(amount > 0, E_ZERO_AMOUNT);

        let reqs = borrow_global_mut<Requests>(@cedra_framework);
        assert!(!table::contains(&reqs.used_nonce, nonce), E_NONCE_USED);
        table::add(&mut reqs.used_nonce, nonce, true);

        let caps = borrow_global<Caps>(@cedra_framework);
        let minted = coin::mint<WETH>(amount, &caps.mint);
        coin::deposit<WETH>(to, minted);

        let evs = borrow_global_mut<BridgeEvents>(@cedra_framework);
        if (features::module_event_migration_enabled()) {
            ev::emit(DepositObserved { eth_tx_hash, to, amount, nonce });
            ev::emit(MintExecuted { to, amount, nonce });
        } else {
            ev::emit_event(&mut evs.deposit_observed, DepositObserved { eth_tx_hash, to, amount, nonce });
            ev::emit_event(&mut evs.mint_executed,   MintExecuted    { to, amount, nonce });
        };
    }

    /* ===================== Withdrawal (user burn but multisig-approved) ===================== */

    /// Multisig pre-approves a specific withdrawal (user, recipient, amount, nonce).
    public entry fun approve_withdrawal(
        multisig: &signer,
        user: address,
        eth_recipient: vector<u8>,
        amount: u64,
        nonce: u64
    ) acquires Admin, Requests, Config {
        assert_not_paused();
        assert_multisig(multisig);
        assert!(amount > 0, E_ZERO_AMOUNT);
        assert!(vector::length(&eth_recipient) == 20, E_BAD_INPUT);

        let reqs = borrow_global_mut<Requests>(@cedra_framework);
        // Cannot re-approve a used nonce
        assert!(!table::contains(&reqs.used_nonce, nonce), E_NONCE_USED);
        // Cannot overwrite an existing approval
        assert!(!table::contains(&reqs.approvals, nonce), E_ALREADY_ACTIVE);

        table::add(
            &mut reqs.approvals,
            nonce,
            WithdrawalApproval { user, eth_recipient, amount }
        );
    }

    /// User executes the withdrawal after multisig approval.
    public entry fun withdraw_to_l1(
        user: &signer,
        eth_recipient: vector<u8>,
        amount: u64,
        nonce: u64
    ) acquires Caps, BridgeEvents, Requests, Config {
        assert_not_paused();
        assert!(amount > 0, E_ZERO_AMOUNT);
        assert!(vector::length(&eth_recipient) == 20, E_BAD_INPUT);

        let reqs = borrow_global_mut<Requests>(@cedra_framework);

        // Must have prior approval
        assert!(table::contains(&reqs.approvals, nonce), E_NO_APPROVAL);
        let appr = table::borrow(&reqs.approvals, nonce);

        // Approval must match caller + params exactly
        assert!(appr.user == signer::address_of(user), E_APPROVAL_MISMATCH);
        assert!(appr.amount == amount, E_APPROVAL_MISMATCH);
        assert!(appr.eth_recipient == eth_recipient, E_APPROVAL_MISMATCH);

        // Burn the user's WETH
        let caps = borrow_global<Caps>(@cedra_framework);
        let coins = coin::withdraw<WETH>(user, amount);
        coin::burn<WETH>(coins, &caps.burn);

        // Mark nonce used and clear approval
        assert!(!table::contains(&reqs.used_nonce, nonce), E_NONCE_USED);
        table::add(&mut reqs.used_nonce, nonce, true);
        // Remove approval to prevent re-use
        let _ = table::remove(&mut reqs.approvals, nonce);

        // Emit event
        let evs = borrow_global_mut<BridgeEvents>(@cedra_framework);
        if (features::module_event_migration_enabled()) {
            ev::emit(Withdrawal { from: signer::address_of(user), eth_recipient, amount, nonce });
        } else {
            ev::emit_event(&mut evs.withdrawal, Withdrawal { from: signer::address_of(user), eth_recipient, amount, nonce });
        };
    }

    /// Users can register CoinStore<WETH> to receive minted coins.
    public entry fun register_weth_store(user: &signer) { coin::register<WETH>(user); }

    /* ===================== Views ===================== */

    #[view]
    public fun admin_multisig(): address acquires Admin {
        borrow_global<Admin>(@cedra_framework).multisig
    }

    #[view]
    public fun nonce_used(n: u64): bool acquires Requests {
        table::contains(&borrow_global<Requests>(@cedra_framework).used_nonce, n)
    }

        /************** TESTS **************/
    #[test_only]
    use cedra_framework::bridge;

    // Mini-genesis for tests: create & init @cedra_framework, then init bridge.
    #[test_only]
    fun setup_framework_for_tests(): signer {
        let (cf, _cap) = account::create_framework_reserved_account(@cedra_framework);
        account::initialize(&cf);
        bridge::initialize(&cf);
        cf
    }

    /// Mint executes when (and only when) the configured multisig calls `execute_deposit`.
    #[test(msig=@0x501, user=@0x2222)]
    public entry fun test_multisig_quorum_mints(msig: &signer, user: &signer)
    acquires Caps, Requests, BridgeEvents, Config, Admin {
        let cf = setup_framework_for_tests();

        // Set the bridge admin to the "multisig" address
        bridge::set_multisig_framework_only(&cf, signer::address_of(msig));

        // User prepares to receive WETH
        bridge::register_weth_store(user);
        let u = signer::address_of(user);
        let before = coin::balance<bridge::WETH>(u);

        // Multisig "executes" the bridge mint after approvals (off-chain / other module)
        let amt = 250;
        let nonce = 42;
        bridge::execute_deposit(msig, u, amt, nonce, b"eth-tx-42");

        let after = coin::balance<bridge::WETH>(u);
        assert!(after == before + amt, 0x1001);
        assert!(bridge::nonce_used(nonce), 0x1002);
    }

    /// Rotating the admin multisig changes who can call the gated functions.
    #[test(msig1=@0xAAA, msig2=@0xBBB, to=@0x3333)]
    public entry fun test_admin_multisig_rotation(msig1: &signer, msig2: &signer, to: &signer)
    acquires Caps, Requests, BridgeEvents, Config, Admin {
        let cf = setup_framework_for_tests();

        bridge::set_multisig_framework_only(&cf, signer::address_of(msig1));
        bridge::register_weth_store(to);

        let to_addr = signer::address_of(to);
        let b0 = coin::balance<bridge::WETH>(to_addr);

        // Works for msig1
        bridge::execute_deposit(msig1, to_addr, 100, 1, b"tx-1");

        // Rotate to msig2
        bridge::rotate_multisig(msig1, signer::address_of(msig2));
        assert!(bridge::admin_multisig() == signer::address_of(msig2), 0x1101);

        // Now only msig2 should work
        bridge::execute_deposit(msig2, to_addr, 200, 2, b"tx-2");

        let b1 = coin::balance<bridge::WETH>(to_addr);
        assert!(b1 == b0 + 100 + 200, 0x1102);
    }

    /// Paused blocks execute_deposit.
    #[test(msig=@0x501, to=@0x4444)]
    #[expected_failure(abort_code = 11)] // E_PAUSED
    public entry fun test_pause_blocks_paths_execute_deposit(msig: &signer, to: &signer)
    acquires Caps, Requests, BridgeEvents, Config, Admin {
        let cf = setup_framework_for_tests();

        bridge::set_multisig_framework_only(&cf, signer::address_of(msig));
        bridge::register_weth_store(to);

        bridge::pause(msig);
        bridge::execute_deposit(msig, signer::address_of(to), 10, 9, b"tx-paused");
    }

    /// Paused also blocks withdrawals (even with approval).
    #[test(msig=@0x501, user=@0x7777)]
    #[expected_failure(abort_code = 11)] // E_PAUSED
    public entry fun test_pause_blocks_paths_withdraw(msig: &signer, user: &signer)
    acquires Caps, Requests, BridgeEvents, Config, Admin {
        let cf = setup_framework_for_tests();

        bridge::set_multisig_framework_only(&cf, signer::address_of(msig));
        bridge::register_weth_store(user);

        // Give the user some WETH so a withdrawal would otherwise succeed
        bridge::execute_deposit(msig, signer::address_of(user), 100, 1001, b"tx-mint");

        // Normally: multisig approves -> user withdraws.
        // But pause first; withdraw must abort E_PAUSED regardless of approval.
        bridge::pause(msig);

        // (Approval would also be blocked, but we don't need it E_PAUSED triggers first.)
        bridge::withdraw_to_l1(user, b"00000000000000000000", 50, 9001);
    }

    /// Full withdrawal happy path: multisig approves and user withdraws; nonce is consumed; approval is removed.
    #[test(msig=@0x501, user=@0x201)]
    public entry fun test_withdrawal_approval_and_execution(msig: &signer, user: &signer)
    acquires Caps, Requests, BridgeEvents, Config, Admin {
        let cf = setup_framework_for_tests();

        bridge::set_multisig_framework_only(&cf, signer::address_of(msig));
        bridge::register_weth_store(user);
        let u = signer::address_of(user);

        // Fund user
        bridge::execute_deposit(msig, u, 1000, 7001, b"tx-fund");

        let before = coin::balance<bridge::WETH>(u);

        // Approve specific withdrawal => (user, recipient, amount, nonce)
        let recp = b"11111111111111111111"; // 20 bytes
        let amount = 150;
        let nonce = 9001;
        bridge::approve_withdrawal(msig, u, recp, amount, nonce);

        // User executes
        bridge::withdraw_to_l1(user, recp, amount, nonce);

        let after = coin::balance<bridge::WETH>(u);
        assert!(before == after + amount, 0x1201);
        assert!(bridge::nonce_used(nonce), 0x1202);
        // Re-using same nonce would now fail (approval removed + nonce used) in a separate failure test.
    }
}