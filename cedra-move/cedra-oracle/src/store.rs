use cedra_sdk::{
    transaction_builder::{cedra_stdlib, TransactionFactory},
    transaction::{authenticator::AuthenticationKey, SignedTransaction, TransactionPayload},
}

// build and run transcation factory to store price list in the blockchain.
// SetPrice , Remove price from move.