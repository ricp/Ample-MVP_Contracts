use crate::*;
use near_sdk::ext_contract;

/// Gas amount necessary to call ft_transfer on different contracts
pub const FT_TRANSFER_GAS: Gas = Gas(50_000_000_000_000);
/// Gas amount necessary to call resolve_reward_transfer on self
/// as a callback
pub const REWARD_WITHDRAW_CALLBACK_GAS: Gas = Gas(50_000_000_000_000);

/// Interface to call cross contract method on
/// NEP-141 adherent contracts.
#[ext_contract(ext_ft)]
pub trait FungibleToken {
    fn ft_transfer(receiver_id: AccountId, amount: U128, memo: Option<String>);
}

/// Interface to call callbacks on the contract itself
#[ext_contract(ext_self)]
pub trait RewardsCallback {
    fn resolve_reward_transfer(receiver_id: AccountId, amount: U128);
}