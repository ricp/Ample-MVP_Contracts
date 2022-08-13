use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct RpsManager {
    pub account_rps: U128,
    pub rewards_received: U128,
    pub rewards_balance: U128,
}

impl RpsManager {
    pub fn new(contract_rps: u128) -> Self {
        Self {
            account_rps: U128(contract_rps),
            rewards_received: U128(0),
            rewards_balance: U128(0),
        }
    }
    pub fn update_rps(&mut self, contract_rps: u128, token_balance: u128) {
        let rps_diff = contract_rps - self.account_rps.0;
        let new_rewards = token_balance * rps_diff;
        self.account_rps = U128(contract_rps);
        self.rewards_received = U128(self.rewards_received.0 + new_rewards);
        self.rewards_balance = U128(self.rewards_balance.0 + new_rewards);
    }
}
