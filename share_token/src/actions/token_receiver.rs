//! token_receiver module
//! 
//! Implements NEP-141 ft_on_transfer method to handle receival of
//! tokens by the contract.

use crate::*;

impl Contract {
    /// Allows the transfer of the reward token to the contract and its immediate
    /// distribution among share owners in the proportion of their ownership
    #[allow(unused_variables)]
    pub fn ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String) -> U128 {
        assert_eq!(
            self.reward_token,
            env::predecessor_account_id(),
            "Invalid reward token, can only tranfer token: {}",
            self.reward_token
        );

        match msg.as_str() {
            "deposit_profits" => {
                let total_reward_tokens = self.reward_tokens_all_time_count.0 + amount.0;
                self.reward_tokens_all_time_count = U128(total_reward_tokens);
                self.contract_rps = U128(total_reward_tokens / self.ft_functionality.ft_total_supply().0);
                U128(0)
            }
            _ => panic!("Invalid msg param"),
        }
    }
}
