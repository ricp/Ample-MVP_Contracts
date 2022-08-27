//! Rewards actions module
//!
//! Allows users to claim their received rewards or
//! check their current reward amount

use crate::ext_interface::{ext_ft, ext_self, FT_TRANSFER_GAS, REWARD_WITHDRAW_CALLBACK_GAS};
use crate::*;
use near_sdk::is_promise_success;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn claim_rewards(&mut self) -> PromiseOrValue<bool> {
        let account_id = env::predecessor_account_id();
        let withdraw_value = self.withdraw_rewards(&account_id);

        if withdraw_value == U128(0) {
            PromiseOrValue::Value(false)
        } else {
            PromiseOrValue::Promise(
                ext_ft::ext(self.reward_token.clone())
                    .with_static_gas(FT_TRANSFER_GAS)
                    .with_attached_deposit(1)
                    .ft_transfer(account_id.clone(), withdraw_value, None)
                    .then(
                        ext_self::ext(env::current_account_id())
                            .with_static_gas(REWARD_WITHDRAW_CALLBACK_GAS)
                            .resolve_reward_transfer(account_id, withdraw_value),
                    ),
            )
        }
    }

    #[private]
    pub fn resolve_reward_transfer(&mut self, receiver_id: AccountId, amount: U128) {
        if !is_promise_success() {
            self.rollback_withdraw_reward(&receiver_id, amount.0);
        }
    }

    pub fn view_claimable_rewards(&self, account_id: AccountId) -> U128 {
        let mut user_rps = self
            .accounts_rps
            .get(&account_id)
            .unwrap_or(RpsManager::new(self.contract_rps.0));

        let user_balance = self.ft_functionality.ft_balance_of(account_id.clone());

        user_rps.update_rps(self.contract_rps.0, user_balance.0);
        user_rps.rewards_balance
    }
}
