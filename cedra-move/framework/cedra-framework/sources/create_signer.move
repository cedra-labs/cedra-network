/// Provides a common place for exporting `create_signer` across the Cedra Framework.
///
/// To use create_signer, add the module below, such that:
/// `friend cedra_framework::friend_wants_create_signer`
/// where `friend_wants_create_signer` is the module that needs `create_signer`.
///
/// Note, that this is only available within the Cedra Framework.
///
/// This exists to make auditing straight forward and to limit the need to depend
/// on account to have access to this.
module cedra_framework::create_signer {
    friend cedra_framework::account;
    friend cedra_framework::cedra_account;
    friend cedra_framework::coin;
    friend cedra_framework::fungible_asset;
    friend cedra_framework::genesis;
    friend cedra_framework::account_abstraction;
    friend cedra_framework::multisig_account;
    friend cedra_framework::object;
    friend cedra_framework::permissioned_signer;
    friend cedra_framework::transaction_validation;
    friend cedra_framework::bridge;

    public(friend) native fun create_signer(addr: address): signer;
}
