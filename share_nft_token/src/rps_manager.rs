use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;

/// Keeps track of each user's rewards received and claimed.
#[derive(BorshDeserialize, BorshSerialize)]
pub struct RpsManager {
    pub account_rps_token: U128,
    pub rewards_received_token: U128,
    pub rewards_balance_token: U128,

    pub account_rps_near: U128,
    pub rewards_received_near: U128,
    pub rewards_balance_near: U128,
}

impl RpsManager {
    /// Initializes new user. should pass current contract_rps
    /// as argument to correctly account for their rewards.
    pub fn new(contract_rps_token: u128, contract_rps_near: u128) -> Self {
        Self {
            account_rps_token: U128(contract_rps_token),
            rewards_received_token: U128(0),
            rewards_balance_token: U128(0),

            account_rps_near: U128(contract_rps_near),
            rewards_received_near: U128(0),
            rewards_balance_near: U128(0),
        }
    }

    /// Distributes rewards to an user after interaction based on
    /// the contract's current rps and the user's share balance.
    pub fn update_rps(&mut self, contract_rps_token: u128, contract_rps_near: u128, user_token_balance: u128) {
        let rps_diff_token = contract_rps_token - self.account_rps_token.0;
        let new_rewards_token = user_token_balance * rps_diff_token;
        self.account_rps_token = U128(contract_rps_token);
        self.rewards_received_token = U128(self.rewards_received_token.0 + new_rewards_token);
        self.rewards_balance_token = U128(self.rewards_balance_token.0 + new_rewards_token);

        let rps_diff_near = contract_rps_near - self.account_rps_near.0;
        let new_rewards_near = user_token_balance * rps_diff_near;
        self.account_rps_near = U128(contract_rps_near);
        self.rewards_received_near = U128(self.rewards_received_near.0 + new_rewards_near);
        self.rewards_balance_near = U128(self.rewards_received_near.0 + new_rewards_near);
    }

    /// Zeros the account's reward balance and returns its value
    pub fn withdraw_rewards(&mut self) -> (U128, U128) {
        let transfer_balance_token = self.rewards_balance_token;
        let transfer_balance_near = self.rewards_balance_near;
        self.rewards_balance_token = U128(0);
        self.rewards_balance_near = U128(0);
        (transfer_balance_token, transfer_balance_near)
    }
}
