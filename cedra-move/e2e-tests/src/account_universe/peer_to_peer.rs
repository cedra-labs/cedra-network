// Copyright © Cedra Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_universe::{AUTransactionGen, AccountPair, AccountPairGen, AccountUniverse},
    common_transactions::peer_to_peer_txn,
};
use cedra_types::{
    transaction::{ExecutionStatus, SignedTransaction, TransactionStatus},
    vm_status::StatusCode,
};
use move_core_types::{
    ident_str,
    language_storage::{ModuleId, CORE_CODE_ADDRESS},
    vm_status::AbortLocation,
};
use proptest::prelude::*;
use proptest_derive::Arbitrary;
use std::sync::Arc;

/// Represents a peer-to-peer transaction performed in the account universe.
///
/// The parameters are the minimum and maximum balances to transfer.
#[derive(Arbitrary, Clone, Debug)]
#[proptest(params = "(u64, u64)")]
pub struct P2PTransferGen {
    sender_receiver: AccountPairGen,
    #[proptest(strategy = "params.0 ..= params.1")]
    amount: u64,
}

impl AUTransactionGen for P2PTransferGen {
    fn apply(
        &self,
        universe: &mut AccountUniverse,
    ) -> (SignedTransaction, (TransactionStatus, u64)) {
        let AccountPair {
            account_1: sender,
            account_2: receiver,
            ..
        } = self.sender_receiver.pick(universe);

        let txn = peer_to_peer_txn(
            sender.account(),
            receiver.account(),
            sender.sequence_number,
            self.amount,
            1, // sets unit gas price, ensures an aggregator is used for total supply.
        );

        // Now figure out whether the transaction will actually work.
        // This means that we'll get through the main part of the transaction.
        let enough_to_transfer = sender.balance >= self.amount;
        let gas_amount = sender.peer_to_peer_gas_cost() * txn.gas_unit_price();
        let to_deduct = self.amount + gas_amount;
        let enough_max_gas = sender.balance >= gas_amount;
        let mut gas_used = 0;
        // This means that we'll get through the entire transaction, including the epilogue
        // (where gas costs are deducted).
        let enough_to_succeed = sender.balance >= to_deduct;

        // Expect a failure if the amount is greater than the current balance.
        // XXX return the failure somehow?
        let status;
        match (enough_max_gas, enough_to_transfer, enough_to_succeed) {
            (true, true, true) => {
                // Success!
                sender.sequence_number += 1;
                sender.balance -= to_deduct;

                receiver.balance += self.amount;

                status = TransactionStatus::Keep(ExecutionStatus::Success);
                gas_used = sender.peer_to_peer_gas_cost();
            },
            (true, true, false) => {
                // Enough gas to pass validation and to do the transfer, but not enough to succeed
                // in the epilogue. The transaction will be run and gas will be deducted from the
                // sender, but no other changes will happen.
                sender.sequence_number += 1;
                gas_used = sender.peer_to_peer_gas_cost();
                sender.balance -= gas_used * txn.gas_unit_price();
                // the balance was insufficient while trying to deduct gas costs in the
                // epilogue.
                // TODO: define these values in a central location
                status = TransactionStatus::Keep(ExecutionStatus::MoveAbort {
                    location: AbortLocation::Module(ModuleId::new(
                        CORE_CODE_ADDRESS,
                        ident_str!("coin").to_owned(),
                    )),
                    code: 65542,
                    info: None,
                });
            },
            (true, false, _) => {
                // Enough to pass validation but not to do the transfer. The transaction will be run
                // and gas will be deducted from the sender, but no other changes will happen.
                sender.sequence_number += 1;
                gas_used = sender.peer_to_peer_too_low_gas_cost();
                sender.balance -= gas_used * txn.gas_unit_price();
                // the balance was insufficient while trying to transfer.
                status = TransactionStatus::Keep(ExecutionStatus::MoveAbort {
                    location: AbortLocation::Module(ModuleId::new(
                        CORE_CODE_ADDRESS,
                        ident_str!("coin").to_owned(),
                    )),
                    code: 65542,
                    info: None,
                });
            },
            (false, _, _) => {
                // Not enough gas to pass validation. Nothing will happen.
                status = TransactionStatus::Discard(
                    StatusCode::INSUFFICIENT_BALANCE_FOR_TRANSACTION_FEE,
                );
            },
        }

        (txn, (status, gas_used))
    }
}

pub fn p2p_strategy(
    min: u64,
    max: u64,
) -> impl Strategy<Value = Arc<dyn AUTransactionGen + 'static>> {
    prop_oneof![
        3 => any_with::<P2PTransferGen>((min, max)).prop_map(P2PTransferGen::arced),
    ]
}
