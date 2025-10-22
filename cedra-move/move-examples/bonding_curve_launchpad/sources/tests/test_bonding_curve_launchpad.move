#[test_only]
module bonding_curve_launchpad::test_bonding_curve_launchpad {
    use cedra_std::string;
    use cedra_std::signer;
    use cedra_std::math64;
    use cedra_framework::account;
    use cedra_framework::coin;
    use cedra_framework::cedra_coin::{Self, CedraCoin};
    use cedra_framework::primary_fungible_store;
    use bonding_curve_launchpad::bonding_curve_launchpad;
    use bonding_curve_launchpad::liquidity_pairs;
    use swap::test_helpers;

    const ELIQUIDITY_PAIR_SWAP_AMOUNTOUT_INCORRECT: u64 = 1001;
    const EUSER_CEDRA_BALANCE_INCORRECT: u64 = 10001;
    const EINCORRECT_FROZEN_STATUS: u64 = 10002;
    const EUSER_FA_BALANCE_INCORRECT: u64 = 10003;

    //---------------------------Test Helpers---------------------------
    fun test_setup_accounts(
        cedra_framework: &signer, _swap_dex_signer: &signer, bonding_curve_creator: &signer
    ) {
        account::create_account_for_test(@0x1);
        account::create_account_for_test(@0xcafe);
        account::create_account_for_test(@bonding_curve_launchpad);
        account::create_account_for_test(@0x803);
        coin::register<CedraCoin>(bonding_curve_creator);

        let (burn_cap, mint_cap) = cedra_coin::initialize_for_test(cedra_framework);
        let bcc_coins = coin::mint(1_000_000_000_000_000, &mint_cap);
        let bcc_address = signer::address_of(bonding_curve_creator);
        coin::deposit(bcc_address, bcc_coins);

        coin::destroy_burn_cap(burn_cap);
        coin::destroy_mint_cap(mint_cap);
    }

    fun test_setup_initialize_contracts(
        swap_dex_signer: &signer, bcl_owner_signer: &signer
    ) {
        test_helpers::set_up(swap_dex_signer);
        liquidity_pairs::initialize_for_test(bcl_owner_signer);
        bonding_curve_launchpad::initialize_for_test(bcl_owner_signer);
    }

    //---------------------------Unit Tests---------------------------
    #[test(deployer = @bonding_curve_launchpad)]
    #[expected_failure(abort_code = liquidity_pairs::ELIQUIDITY_PAIR_DOES_NOT_EXIST, location = liquidity_pairs)]
    public fun test_nonexistant_is_frozen(deployer: &signer) {
        account::create_account_for_test(@0x1);
        liquidity_pairs::initialize_for_test(deployer);
        bonding_curve_launchpad::initialize_for_test(deployer);
        let name = string::utf8(b"SheepyCoin");
        let symbol = string::utf8(b"SHEEP");
        bonding_curve_launchpad::get_is_frozen(name, symbol);
    }

    #[test(deployer = @bonding_curve_launchpad)]
    #[expected_failure(abort_code = 393218, location = cedra_framework::object)]
    public fun test_nonexistant_get_metadata(deployer: &signer) {
        account::create_account_for_test(@0x1);
        liquidity_pairs::initialize_for_test(deployer);
        bonding_curve_launchpad::initialize_for_test(deployer);
        let name = string::utf8(b"SheepyCoin");
        let symbol = string::utf8(b"SHEEP");
        bonding_curve_launchpad::get_metadata(name, symbol);
    }

    //---------------------------E2E Tests---------------------------
    #[test(
        cedra_framework = @0x1,
        swap_dex_signer = @0xcafe,
        bcl_owner_signer = @bonding_curve_launchpad,
        bonding_curve_creator = @0x803
    )]
    fun test_e2e_bonding_curve_creation(
        cedra_framework: &signer,
        swap_dex_signer: &signer,
        bcl_owner_signer: &signer,
        bonding_curve_creator: &signer
    ) {
        test_setup_accounts(cedra_framework, swap_dex_signer, bonding_curve_creator);
        test_setup_initialize_contracts(swap_dex_signer, bcl_owner_signer);
        // Create FA and LiquidityPair, w.o Initial Swap.
        let user_address = signer::address_of(bonding_curve_creator);
        let starting_cedra_balance = coin::balance<CedraCoin>(user_address);
        let name = string::utf8(b"SheepyCoin");
        let symbol = string::utf8(b"SHEEP");
        bonding_curve_launchpad::create_fa_pair(
            bonding_curve_creator,
            0,
            name,
            symbol,
            803_000_000,
            8,
            string::utf8(
                b"https://t4.ftcdn.net/jpg/03/12/95/13/360_F_312951336_8LxW7gBLHslTnpbOAwxFo5FpD2R5vGxu.jpg"
            ),
            string::utf8(
                b"https://t4.ftcdn.net/jpg/03/12/95/13/360_F_312951336_8LxW7gBLHslTnpbOAwxFo5FpD2R5vGxu.jpg"
            )
        );
        assert!(coin::balance<CedraCoin>(user_address) == starting_cedra_balance, EUSER_CEDRA_BALANCE_INCORRECT);
        assert!(bonding_curve_launchpad::get_balance(name, symbol, user_address) == 0, EUSER_FA_BALANCE_INCORRECT);
    }

    #[test(
        cedra_framework = @0x1,
        swap_dex_signer = @0xcafe,
        bcl_owner_signer = @bonding_curve_launchpad,
        bonding_curve_creator = @0x803
    )]
    fun test_e2e_bonding_curve_creation_with_initial_liquidity(
        cedra_framework: &signer,
        swap_dex_signer: &signer,
        bcl_owner_signer: &signer,
        bonding_curve_creator: &signer
    ) {
        test_setup_accounts(cedra_framework, swap_dex_signer, bonding_curve_creator);
        test_setup_initialize_contracts(swap_dex_signer, bcl_owner_signer);
        // Create FA and LiquidityPair, w/ Initial Swap.
        let user_address = signer::address_of(bonding_curve_creator);
        let starting_cedra_balance = coin::balance<CedraCoin>(user_address);
        let name = string::utf8(b"SheepyCoin");
        let symbol = string::utf8(b"SHEEP");
        bonding_curve_launchpad::create_fa_pair(
            bonding_curve_creator,
            1_000,
            name,
            symbol,
            803_000_000,
            8,
            string::utf8(
                b"https://t4.ftcdn.net/jpg/03/12/95/13/360_F_312951336_8LxW7gBLHslTnpbOAwxFo5FpD2R5vGxu.jpg"
            ),
            string::utf8(
                b"https://t4.ftcdn.net/jpg/03/12/95/13/360_F_312951336_8LxW7gBLHslTnpbOAwxFo5FpD2R5vGxu.jpg"
            )
        );
        assert!(coin::balance<CedraCoin>(user_address) == starting_cedra_balance - 1000, EUSER_CEDRA_BALANCE_INCORRECT);
        assert!(bonding_curve_launchpad::get_balance(name, symbol, user_address) == 16, EUSER_FA_BALANCE_INCORRECT);
    }

    #[test(
        cedra_framework = @0x1,
        swap_dex_signer = @0xcafe,
        bcl_owner_signer = @bonding_curve_launchpad,
        bonding_curve_creator = @0x803
    )]
    fun test_e2e_bonding_curve_creation_multiple(
        cedra_framework: &signer,
        swap_dex_signer: &signer,
        bcl_owner_signer: &signer,
        bonding_curve_creator: &signer
    ) {
        test_setup_accounts(cedra_framework, swap_dex_signer, bonding_curve_creator);
        test_setup_initialize_contracts(swap_dex_signer, bcl_owner_signer);
        // Create FA and LiquidityPair, w.o Initial Swap.
        let name = string::utf8(b"SheepyCoin");
        let symbol = string::utf8(b"SHEEP");
        bonding_curve_launchpad::create_fa_pair(
            bonding_curve_creator,
            0,
            name,
            symbol,
            803_000_000,
            8,
            string::utf8(
                b"https://t4.ftcdn.net/jpg/03/12/95/13/360_F_312951336_8LxW7gBLHslTnpbOAwxFo5FpD2R5vGxu.jpg"
            ),
            string::utf8(
                b"https://t4.ftcdn.net/jpg/03/12/95/13/360_F_312951336_8LxW7gBLHslTnpbOAwxFo5FpD2R5vGxu.jpg"
            )
        );
        let second_fa_name = string::utf8(b"RammyCoin");
        let second_fa_symbol = string::utf8(b"RAM");
        bonding_curve_launchpad::create_fa_pair(
            bonding_curve_creator,
            0,
            second_fa_name,
            second_fa_symbol,
            803_000_000,
            8,
            string::utf8(
                b"https://t4.ftcdn.net/jpg/03/12/95/13/360_F_312951336_8LxW7gBLHslTnpbOAwxFo5FpD2R5vGxu.jpg"
            ),
            string::utf8(
                b"https://t4.ftcdn.net/jpg/03/12/95/13/360_F_312951336_8LxW7gBLHslTnpbOAwxFo5FpD2R5vGxu.jpg"
            )
        );
    }

    #[test(
        cedra_framework = @0x1,
        swap_dex_signer = @0xcafe,
        bcl_owner_signer = @bonding_curve_launchpad,
        bonding_curve_creator = @0x803
    )]
    fun test_e2e_directional_swaps(
        cedra_framework: &signer,
        swap_dex_signer: &signer,
        bcl_owner_signer: &signer,
        bonding_curve_creator: &signer
    ) {
        test_e2e_bonding_curve_creation(cedra_framework, swap_dex_signer, bcl_owner_signer, bonding_curve_creator);
        let user_address = signer::address_of(bonding_curve_creator);
        let name = string::utf8(b"SheepyCoin");
        let symbol = string::utf8(b"SHEEP");
        let starting_cedra_balance = coin::balance<CedraCoin>(user_address);
        // Cedra -> FA
        bonding_curve_launchpad::swap(bonding_curve_creator, name, symbol, false, 100_000_000);
        assert!(
            coin::balance<CedraCoin>(user_address) == starting_cedra_balance - 100_000_000,
            EUSER_CEDRA_BALANCE_INCORRECT
        );
        assert!(
            bonding_curve_launchpad::get_balance(name, symbol, user_address) == 1_602_794,
            ELIQUIDITY_PAIR_SWAP_AMOUNTOUT_INCORRECT
        );
        // FA -> Cedra
        bonding_curve_launchpad::swap(bonding_curve_creator, name, symbol, true, 1_602_794);
        assert!(
            coin::balance<CedraCoin>(user_address) == starting_cedra_balance - 26,
            EUSER_CEDRA_BALANCE_INCORRECT
        ); // u256/u64 precision loss.
        assert!(
            bonding_curve_launchpad::get_balance(name, symbol, user_address) == 0,
            ELIQUIDITY_PAIR_SWAP_AMOUNTOUT_INCORRECT
        );
    }

    #[test(
        cedra_framework = @0x1,
        swap_dex_signer = @0xcafe,
        bcl_owner_signer = @bonding_curve_launchpad,
        bonding_curve_creator = @0x803
    )]
    fun test_e2e_graduation(
        cedra_framework: &signer,
        swap_dex_signer: &signer,
        bcl_owner_signer: &signer,
        bonding_curve_creator: &signer
    ) {
        test_e2e_bonding_curve_creation(cedra_framework, swap_dex_signer, bcl_owner_signer, bonding_curve_creator);
        let grad_cedra: u64 = 6_000 * math64::pow(10, (8 as u64));
        let name = string::utf8(b"SheepyCoin");
        let symbol = string::utf8(b"SHEEP");
        assert!(bonding_curve_launchpad::get_is_frozen(name, symbol) == true, EINCORRECT_FROZEN_STATUS);
        bonding_curve_launchpad::swap(
            bonding_curve_creator,
            name,
            symbol,
            false,
            grad_cedra
        ); // Over-threshold Swap. Cedra -> FA
        assert!(bonding_curve_launchpad::get_is_frozen(name, symbol) == false, EINCORRECT_FROZEN_STATUS);
    }

    fun test_e2e_swap_after_graduation(
        cedra_framework: &signer,
        swap_dex_signer: &signer,
        bcl_owner_signer: &signer,
        bonding_curve_creator: &signer
    ) {
        test_e2e_graduation(cedra_framework, swap_dex_signer, bcl_owner_signer, bonding_curve_creator);
        let fa_obj_metadata =
            bonding_curve_launchpad::get_metadata(
                string::utf8(b"SheepyCoin"),
                string::utf8(b"SHEEP")
            );
        primary_fungible_store::transfer(bonding_curve_creator, fa_obj_metadata, @0xcafe, 100);
    }

    // ----E2E EXPECTED FAILING-----
    #[test(
        cedra_framework = @0x1,
        swap_dex_signer = @0xcafe,
        bcl_owner_signer = @bonding_curve_launchpad,
        bonding_curve_creator = @0x803
    )]
    #[expected_failure(abort_code = bonding_curve_launchpad::bonding_curve_launchpad::EFA_EXISTS_ALREADY, location = bonding_curve_launchpad)]
    fun test_e2e_failing_duplicate_FA(
        cedra_framework: &signer,
        swap_dex_signer: &signer,
        bcl_owner_signer: &signer,
        bonding_curve_creator: &signer
    ) {
        test_e2e_bonding_curve_creation(
            cedra_framework,
            swap_dex_signer,
            bcl_owner_signer,
            bonding_curve_creator
        ); // SheepyCoin, SHEEP
        let name = string::utf8(b"SheepyCoin");
        let symbol = string::utf8(b"SHEEP");
        bonding_curve_launchpad::create_fa_pair(
            bonding_curve_creator,
            1_000,
            name,
            symbol,
            803_000_000,
            8,
            string::utf8(
                b"https://t4.ftcdn.net/jpg/03/12/95/13/360_F_312951336_8LxW7gBLHslTnpbOAwxFo5FpD2R5vGxu.jpg"
            ),
            string::utf8(
                b"https://t4.ftcdn.net/jpg/03/12/95/13/360_F_312951336_8LxW7gBLHslTnpbOAwxFo5FpD2R5vGxu.jpg"
            )
        );
    }

    #[test(
        cedra_framework = @0x1,
        swap_dex_signer = @0xcafe,
        bcl_owner_signer = @bonding_curve_launchpad,
        bonding_curve_creator = @0x803
    )]
    #[expected_failure(abort_code = liquidity_pairs::ELIQUIDITY_PAIR_DISABLED, location = liquidity_pairs)]
    fun test_e2e_failing_cedra_swap_after_graduation(
        cedra_framework: &signer,
        swap_dex_signer: &signer,
        bcl_owner_signer: &signer,
        bonding_curve_creator: &signer
    ) {
        test_e2e_graduation(cedra_framework, swap_dex_signer, bcl_owner_signer, bonding_curve_creator);
        bonding_curve_launchpad::swap(
            bonding_curve_creator,
            string::utf8(b"SheepyCoin"),
            string::utf8(b"SHEEP"),
            false,
            1_000_000
        ); // Cedra -> FA
    }

    #[test(
        cedra_framework = @0x1,
        swap_dex_signer = @0xcafe,
        bcl_owner_signer = @bonding_curve_launchpad,
        bonding_curve_creator = @0x803
    )]
    #[expected_failure(abort_code = liquidity_pairs::ELIQUIDITY_PAIR_DISABLED, location = liquidity_pairs)]
    fun test_e2e_failing_fa_swap_after_graduation(
        cedra_framework: &signer,
        swap_dex_signer: &signer,
        bcl_owner_signer: &signer,
        bonding_curve_creator: &signer
    ) {
        test_e2e_graduation(cedra_framework, swap_dex_signer, bcl_owner_signer, bonding_curve_creator);
        bonding_curve_launchpad::swap(
            bonding_curve_creator,
            string::utf8(b"SheepyCoin"),
            string::utf8(b"SHEEP"),
            false,
            10
        ); // FA -> Cedra
    }

    #[test(
        cedra_framework = @0x1,
        swap_dex_signer = @0xcafe,
        bcl_owner_signer = @bonding_curve_launchpad,
        bonding_curve_creator = @0x803
    )]
    #[expected_failure(abort_code = 393218, location = cedra_framework::object)]
    fun test_e2e_failing_swap_of_nonexistant_fa(
        cedra_framework: &signer,
        swap_dex_signer: &signer,
        bcl_owner_signer: &signer,
        bonding_curve_creator: &signer
    ) {
        test_setup_accounts(cedra_framework, swap_dex_signer, bonding_curve_creator);
        test_setup_initialize_contracts(swap_dex_signer, bcl_owner_signer);
        bonding_curve_launchpad::swap(
            bonding_curve_creator,
            string::utf8(b"SheepyCoin"),
            string::utf8(b"SHEEP"),
            false,
            1_000_000
        );
    }

    #[test(
        cedra_framework = @0x1,
        swap_dex_signer = @0xcafe,
        bcl_owner_signer = @bonding_curve_launchpad,
        bonding_curve_creator = @0x803
    )]
    #[expected_failure(abort_code = bonding_curve_launchpad::bonding_curve_launchpad::EFA_FROZEN, location = bonding_curve_launchpad)]
    fun test_e2e_failing_transfer_of_frozen_fa(
        cedra_framework: &signer,
        swap_dex_signer: &signer,
        bcl_owner_signer: &signer,
        bonding_curve_creator: &signer
    ) {
        test_e2e_bonding_curve_creation_with_initial_liquidity(
            cedra_framework,
            swap_dex_signer,
            bcl_owner_signer,
            bonding_curve_creator
        );
        let fa_obj_metadata = bonding_curve_launchpad::get_metadata(
            string::utf8(b"SheepyCoin"),
            string::utf8(b"SHEEP")
        );
        primary_fungible_store::transfer(bonding_curve_creator, fa_obj_metadata, @0xcafe, 10);
    }

    #[test(
        cedra_framework = @0x1,
        swap_dex_signer = @0xcafe,
        bcl_owner_signer = @bonding_curve_launchpad,
        bonding_curve_creator = @0x803
    )]
    #[expected_failure(abort_code = bonding_curve_launchpad::bonding_curve_launchpad::ELIQUIDITY_PAIR_SWAP_AMOUNTIN_INVALID, location = bonding_curve_launchpad)]
    fun test_e2e_failing_swap_of_zero_input_cedra(
        cedra_framework: &signer,
        swap_dex_signer: &signer,
        bcl_owner_signer: &signer,
        bonding_curve_creator: &signer
    ) {
        test_e2e_bonding_curve_creation(cedra_framework, swap_dex_signer, bcl_owner_signer, bonding_curve_creator);
        bonding_curve_launchpad::swap(
            bonding_curve_creator,
            string::utf8(b"SheepyCoin"),
            string::utf8(b"SHEEP"),
            false,
            0
        ); // Cedra -> FA
    }

    #[test(
        cedra_framework = @0x1,
        swap_dex_signer = @0xcafe,
        bcl_owner_signer = @bonding_curve_launchpad,
        bonding_curve_creator = @0x803
    )]
    #[expected_failure(abort_code = bonding_curve_launchpad::bonding_curve_launchpad::ELIQUIDITY_PAIR_SWAP_AMOUNTIN_INVALID, location = bonding_curve_launchpad)]
    fun test_e2e_failing_swap_of_zero_input_fa(
        cedra_framework: &signer,
        swap_dex_signer: &signer,
        bcl_owner_signer: &signer,
        bonding_curve_creator: &signer
    ) {
        test_e2e_bonding_curve_creation(cedra_framework, swap_dex_signer, bcl_owner_signer, bonding_curve_creator);
        bonding_curve_launchpad::swap(
            bonding_curve_creator,
            string::utf8(b"SheepyCoin"),
            string::utf8(b"SHEEP"),
            false,
            0
        ); // Swap afer graduation, guaranteed to fail. FA -> Cedra
    }

    #[test(
        cedra_framework = @0x1,
        swap_dex_signer = @0xcafe,
        bcl_owner_signer = @bonding_curve_launchpad,
        bonding_curve_creator = @0x803
    )]
    #[expected_failure(abort_code = liquidity_pairs::EFA_PRIMARY_STORE_DOES_NOT_EXIST, location = liquidity_pairs)]
    fun test_e2e_failing_swap_of_user_without_fa(
        cedra_framework: &signer,
        swap_dex_signer: &signer,
        bcl_owner_signer: &signer,
        bonding_curve_creator: &signer
    ) {
        test_e2e_bonding_curve_creation(cedra_framework, swap_dex_signer, bcl_owner_signer, bonding_curve_creator);
        bonding_curve_launchpad::swap(
            bonding_curve_creator,
            string::utf8(b"SheepyCoin"),
            string::utf8(b"SHEEP"),
            true,
            10000
        ); // Swap afer graduation, guaranteed to fail. FA -> Cedra
    }
}
