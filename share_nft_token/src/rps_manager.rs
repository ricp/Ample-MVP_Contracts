use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;

/// Keeps track of each user's rewards received and claimed.
#[derive(BorshDeserialize, BorshSerialize)]
pub struct RpsManager {
    pub account_rps: U128,
    pub rewards_received: U128,
    pub rewards_balance: U128,
}

impl RpsManager {
    /// Initializes new user. should pass current contract_rps
    /// as argument to correctly account for their rewards.
    pub fn new(contract_rps: u128) -> Self {
        Self {
            account_rps: U128(contract_rps),
            rewards_received: U128(0),
            rewards_balance: U128(0),
        }
    }

    /// Distributes rewards to an user after interaction based on
    /// the contract's current rps and the user's share balance.
    pub fn update_rps(&mut self, contract_rps: u128, user_token_balance: u128) {
        let rps_diff = contract_rps - self.account_rps.0;
        let new_rewards = user_token_balance * rps_diff;
        self.account_rps = U128(contract_rps);
        self.rewards_received = U128(self.rewards_received.0 + new_rewards);
        self.rewards_balance = U128(self.rewards_balance.0 + new_rewards);
    }

    /// Zeros the account's reward balance and returns its value
    pub fn withdraw_rewards(&mut self) -> U128 {
        let transfer_balance = self.rewards_balance;
        self.rewards_balance = U128(0);
        transfer_balance
    }
}
