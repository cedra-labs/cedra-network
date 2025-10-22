/// This contract is used to manage the commission rate for the node operator. There are two entities involved:
/// 1. Manager: The account that can set the commission rate and change the operator account.
/// 2. Operator: The account that receives the commission in dollars in exchange for running the node.
///
/// The commission rate is set in dollars and will be used to determine how much Cedra the operator receives.
/// The commission is distributed to the operator and remaining amount to the manager. If there's not enough balance
/// to pay the commission, either commission rate is set too high or Cedra price is low. In this case, the commission
/// debt will be updated and the operator will receive the remaining balance in the next distribution.
///
/// Important notes:
///
/// 1. There are rounding errors that can lead to 1 octa (1e-8 Cedra) and $1 rounding errors on conversions during
/// distribution. Although the commission amount can be adjusted to make up for these rounding errors for operators,
/// developers using this contract can also add decimals to the dollar amount (e.g. 2 decimals) to reduce the rounding
/// errors.
/// 2. In theory this function can be called very often (e.g. once every few seconds) to use rounding errors
/// to "rob" the operator. This has minimum damage in practice when using this contract for node operator commission
/// as it's only paid out once in a while (at least a few days) so balance is zero before then which makes the attack
/// not possible.
/// This issue is also somewhat mitigated by asserting a min balance before distributing. For other uses of this
/// contract, consider raising the minimum balance to minimize rounding errors from frequent distribution calls.
module staking::commission {
    use cedra_framework::account::{Self, SignerCapability};
    use cedra_framework::cedra_account;
    use cedra_framework::cedra_coin::CedraCoin;
    use cedra_framework::coin;
    use cedra_framework::resource_account;
    use cedra_framework::timestamp;
    use cedra_std::math128;
    use cedra_std::math64;
    use staking::oracle;
    use std::signer;
    use cedra_framework::event;

    const INITIAL_COMMISSION_AMOUNT: u64 = 100000;
    const ONE_YEAR_IN_SECONDS: u64 = 31536000;
    const OCTAS_IN_ONE_Cedra: u128 = 100000000; // 1e8
    const MIN_BALANCE_FOR_DISTRIBUTION: u64 = 100000000; // 1 Cedra

    /// Account is not authorized to call this function.
    const EUNAUTHORIZED: u64 = 1;
    /// Contract must have at least the minimum balance required before distributions can happen.
    const EINSUFFICIENT_BALANCE_FOR_DISTRIBUTION: u64 = 2;
    /// The new operator cannot be the same as the old operator.
    const EOPERATOR_SAME_AS_OLD: u64 = 3;

    struct CommissionConfig has key {
        /// The manager of the contract who can set the commission rate.
        manager: address,
        /// The operator who receives the specified commission in dollars in exchange for running the node.
        operator: address,
        /// The yearly commission rate in dollars. Will be used to determine how much Cedra the operator receives.
        yearly_commission_amount: u64,
        /// Used to withdraw commission.
        signer_cap: SignerCapability,
        /// Timestamp for tracking yearly commission.
        last_update_secs: u64,
        /// Amount of debt in dollars owed to the operator due to insufficient amount received from node commission.
        /// This can happen if the commission rate is set too high or Cedra price is too low.
        commission_debt: u64
    }

    #[event]
    struct CommissionConfigUpdated has drop, store {
        manager: address,
        operator: address,
        old_yearly_commission_amount: u64,
        new_yearly_commission_amount: u64
    }

    #[event]
    struct OperatorUpdated has drop, store {
        requester: address,
        manager: address,
        old_operator: address,
        new_operator: address
    }

    #[event]
    struct CommissionDistributed has drop, store {
        manager: address,
        operator: address,
        usd_price: u128,
        commission_amount_cedra: u64,
        manager_amount_cedra: u64,
        commission_debt_usd: u64
    }

    fun init_module(commission_signer: &signer) {
        let signer_cap = resource_account::retrieve_resource_account_cap(commission_signer, @manager);
        move_to(commission_signer, CommissionConfig {
            manager: @manager,
            operator: @operator,
            yearly_commission_amount: INITIAL_COMMISSION_AMOUNT,
            signer_cap,
            last_update_secs: timestamp::now_seconds(),
            commission_debt: 0,
        });
    }

    #[view]
    public fun operator(): address acquires CommissionConfig {
        CommissionConfig[@staking].operator
    }

    #[view]
    public fun yearly_commission_amount(): u64 acquires CommissionConfig {
        CommissionConfig[@staking].yearly_commission_amount
    }

    #[view]
    public fun commission_owed(): u64 acquires CommissionConfig {
        let config = &CommissionConfig[@staking];
        // Commission earned so far = per second commission rate * seconds passed.
        let now_secs = timestamp::now_seconds();
        let seconds_passed = now_secs - config.last_update_secs;
        let commission_earned = math64::mul_div(seconds_passed, config.yearly_commission_amount, ONE_YEAR_IN_SECONDS);

        commission_earned + config.commission_debt
    }

    #[view]
    public fun commission_owed_in_cedra(): u64 acquires CommissionConfig {
        usd_to_cedra(commission_owed())
    }

    /// Can be called by the manager to change the yearly commission amount.
    public entry fun set_yearly_commission_amount(manager: &signer, new_commission: u64) acquires CommissionConfig {
        assert_manager(manager);
        let config = &mut CommissionConfig[@staking];
        let old_yearly_commission_amount = config.yearly_commission_amount;
        config.yearly_commission_amount = new_commission;

        event::emit(CommissionConfigUpdated {
            manager: config.manager,
            operator: config.operator,
            old_yearly_commission_amount,
            new_yearly_commission_amount: new_commission
        });
    }

    /// Can be called by the manager or operator to change the account that receives the commission.
    public entry fun set_operator(account: &signer, new_operator: address) acquires CommissionConfig {
        assert_manager_or_operator(account);
        let config = &mut CommissionConfig[@staking];
        let old_operator = config.operator;
        assert!(old_operator != new_operator, EOPERATOR_SAME_AS_OLD);
        config.operator = new_operator;

        event::emit(OperatorUpdated {
            requester: signer::address_of(account),
            manager: config.manager,
            old_operator,
            new_operator
        });
    }

    /// Distribute the commission to operator and remaining amount to manager.
    /// Can only be called by the manager or operator.
    ///
    /// Note that in theory this function can be called very often (e.g. once every few seconds) to use rounding errors
    /// to "rob" the operator. This has minimum damage in practice when using this contract for node operator commission
    /// as it's only paid out once in a while (at least a few days) so balance is zero before then which makes the attack
    /// not possible.
    /// This issue is also somewhat mitigated by asserting a min balance before distributing. For other uses of this
    /// contract, consider raising the minimum balance to minimize rounding errors from frequent distribution calls.
    public entry fun distribute_commission(account: &signer) acquires CommissionConfig {
        assert_manager_or_operator(account);

        let balance = coin::balance<CedraCoin>(@staking);
        assert!(balance >= MIN_BALANCE_FOR_DISTRIBUTION, EINSUFFICIENT_BALANCE_FOR_DISTRIBUTION);

        // Commission owed so far plus any debt.
        // There can be a rounding error of 1 octa here when converting from USD to Cedra. This is negligible.
        let commission_in_cedra = commission_owed_in_cedra();

        // Only manager or operator can call this function.
        let config = &mut CommissionConfig[@staking];
        config.last_update_secs = timestamp::now_seconds();
        // Commission debt is already included in commission_owed by the commission_owed function.
        config.commission_debt = 0;

        let commission_signer = &account::create_signer_with_capability(&config.signer_cap);
        // If there's not enough balance to pay the commission, either commission rate is set too high or Cedra price is low.
        // Otherwise, pay the operator the commission in Cedra and send remaining balance to the manager.
        if (balance <= commission_in_cedra) {
            // If balance is exactly equal to commission in Cedra, this will set commission_debt to 0.
            let debt_cedra = commission_in_cedra - balance;
            // There can be rounding error here when converting from Cedra to USD. If this is of concern, the amount of
            // commission can be set higher to cover the rounding error.
            config.commission_debt = apt_to_usd(debt_cedra);
        } else {
            let surplus_balance = balance - commission_in_cedra;
            cedra_account::transfer(commission_signer, config.manager, surplus_balance);
        };

        let remaining_balance = coin::balance<CedraCoin>(@staking);
        cedra_account::transfer(commission_signer, config.operator, remaining_balance);

        event::emit(CommissionDistributed {
            manager: config.manager,
            operator: config.operator,
            usd_price: oracle::get_cedra_price(),
            commission_amount_cedra: apt_to_usd(commission_in_cedra),
            manager_amount_cedra: apt_to_usd(remaining_balance),
            commission_debt_usd: config.commission_debt
        });
    }

    inline fun assert_manager(account: &signer) {
        assert!(signer::address_of(account) == CommissionConfig[@staking].manager, EUNAUTHORIZED);
    }

    inline fun assert_manager_or_operator(account: &signer) {
        let config = &CommissionConfig[@staking];
        let account_addr = signer::address_of(account);
        assert!(account_addr == config.manager || account_addr == config.operator, EUNAUTHORIZED);
    }

    inline fun usd_to_cedra(usd_amount: u64): u64 {
        let apt_price = oracle::get_cedra_price();
        // Amount in Cedra octas = amount * number of octas in one Cedra / Cedra price.
        math128::mul_div((usd_amount as u128) * OCTAS_IN_ONE_Cedra, oracle::precision(), apt_price) as u64
    }

    inline fun apt_to_usd(apt_amount: u64): u64 {
        let apt_price = oracle::get_cedra_price();
        // Amount in USD = amount * Cedra price / precision / number of octas in one Cedra.
        math128::mul_div((apt_amount as u128), apt_price, oracle::precision() * OCTAS_IN_ONE_Cedra) as u64
    }

    #[test_only]
    public fun init_for_test(deployer: &signer) {
        let signer_cap = account::create_test_signer_cap(signer::address_of(deployer));
        move_to(deployer, CommissionConfig {
            manager: @manager,
            operator: @operator,
            yearly_commission_amount: INITIAL_COMMISSION_AMOUNT,
            signer_cap,
            last_update_secs: timestamp::now_seconds(),
            commission_debt: 0,
        });
    }
}
