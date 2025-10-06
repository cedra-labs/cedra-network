#[test_only]
module cedra_framework::bridge_tests {
    use std::signer;
    use cedra_framework::bridge;
    use cedra_framework::coin;

    fun setup_three_validators(admin: &signer, v1: &signer, v2: &signer, v3: &signer) {
        bridge::init(admin);
        bridge::initialize_validator(admin, signer::address_of(v1), 5);
        bridge::initialize_validator(admin, signer::address_of(v2), 3);
        bridge::initialize_validator(admin, signer::address_of(v3), 2);
        bridge::join_validator_set(admin, signer::address_of(v1));
        bridge::join_validator_set(admin, signer::address_of(v2));
        bridge::join_validator_set(admin, signer::address_of(v3));

        assert!(bridge::total_weight_for(admin) == 5 + 3 + 2, 0xBEEF);
        
        let a = signer::address_of(admin);
        let (e1, act1, w1, _tw) = bridge::validator_debug(a, signer::address_of(v1));
        let (e2, act2, w2, _tw2) = bridge::validator_debug(a, signer::address_of(v2));
        let (e3, act3, w3, _tw3) = bridge::validator_debug(a, signer::address_of(v3));

        assert!(e1 && act1, 0xD01);
        assert!(e2 && act2, 0xD02);
        assert!(e3 && act3, 0xD03);

        assert!(bridge::is_active_validator_for(admin, signer::address_of(v1)), 0xA1);
        assert!(bridge::is_active_validator_for(admin, signer::address_of(v2)), 0xA2);
        assert!(bridge::is_active_validator_for(admin, signer::address_of(v3)), 0xA3);    }

    #[test(admin=@0xA11CE, v1=@0x101, v2=@0x102, v3=@0x103, to=@0x202)]
    #[expected_failure(abort_code = 10)]
    public entry fun test_execute_mint_requires_quorum(
        admin: &signer, v1: &signer, v2: &signer, v3: &signer, to: &signer
    ) {
        setup_three_validators(admin, v1, v2, v3);
        let _ = v2; let _ = v3;

        let a = signer::address_of(admin);
        let (e1, act1, w1, tw) = bridge::validator_debug(a, signer::address_of(v1));
        let (e2, act2, w2, _)  = bridge::validator_debug(a, signer::address_of(v2));
        let (e3, act3, w3, _)  = bridge::validator_debug(a, signer::address_of(v3));

        assert!(e1, 0xE11); assert!(act1, 0xE12); assert!(w1 == 5, 0xE13);
        assert!(e2, 0xE21); assert!(act2, 0xE22); assert!(w2 == 3, 0xE23);
        assert!(e3, 0xE31); assert!(act3, 0xE32); assert!(w3 == 2, 0xE33);
        assert!(tw == 10, 0xE3F);

        let admin_addr = signer::address_of(admin);
        let to_addr = signer::address_of(to);
        bridge::register_weth_store(to, admin_addr);

        bridge::open_mint_request(admin, to_addr, 100, 7, b"eth-tx-7");

        bridge::approve_mint(v1, admin, 7);
        bridge::execute_mint(admin, admin, 7);
    }

    #[test(admin=@0xA11CE, v1=@0x101, v2=@0x102, v3=@0x103, to=@0x2222)]
    public entry fun test_execute_mint_with_quorum_and_balances(
        admin: &signer, v1: &signer, v2: &signer, v3: &signer, to: &signer
    ) {
        setup_three_validators(admin, v1, v2, v3);

        // define admin_addr BEFORE using it
        let admin_addr = signer::address_of(admin);

        let who = signer::address_of(v1);
        let (e1, act1, w1, tw) = bridge::validator_debug(admin_addr, who);
        bridge::log_balance(admin_addr, b"debug_v1", signer::address_of(v1), if (e1 && act1) { w1 } else { 0 }, tw);
        assert!(e1 && act1, 0xDEAD);

        // prove v1 is active where we think it is
        bridge::log_validator_state(admin_addr, b"pre_approve_v1", signer::address_of(v1));
        assert!(bridge::is_active_validator_for(admin, signer::address_of(v1)), 0);
        let to_addr = signer::address_of(to);

        bridge::register_weth_store(to, admin_addr);

        let amt = 250;
        bridge::open_mint_request(admin, to_addr, amt, 42, b"eth-tx-42");

        bridge::approve_mint(v1, admin, 42);
        bridge::approve_mint(v2, admin, 42);

        let before = coin::balance<bridge::WETH>(to_addr);
        bridge::execute_mint(admin, admin, 42);
        let after  = coin::balance<bridge::WETH>(to_addr);

        bridge::log_balance(admin_addr, b"mint_quorum", to_addr, before, after);
        assert!(after == before + amt, 0);
    }

    #[test(admin=@0xA11CE, v1=@0x101, v2=@0x102, v3=@0x103, user=@0x201)]
    public entry fun test_burn_flow_reduces_balance(
        admin: &signer, v1: &signer, v2: &signer, v3: &signer, user: &signer
    ) {
        setup_three_validators(admin, v1, v2, v3);

        let admin_addr = signer::address_of(admin);
        let user_addr  = signer::address_of(user);
        bridge::register_weth_store(user, admin_addr);

        bridge::open_mint_request(admin, user_addr, 1000, 100, b"eth-tx-100");
        bridge::approve_mint(v1, admin, 100);
        bridge::approve_mint(v2, admin, 100);
        bridge::execute_mint(admin, admin, 100);

        let before = coin::balance<bridge::WETH>(user_addr);
        bridge::burn_for_ethereum_exit(user, admin, b"00000000000000000000", 150, 9001);
        let after = coin::balance<bridge::WETH>(user_addr);

        bridge::log_balance(admin_addr, b"burn_exit", user_addr, before, after);
        assert!(before == after + 150, 0);
    }
}