//! token_receiver module
//!
//! Implements NEP-141 ft_on_transfer method to handle receival of
//! tokens by the contract.

use crate::*;

#[near_bindgen]
impl Contract {
    /// Allows the transfer of NEAR rewards to the contract and its immediate
    /// distribution among share owners in the proportion of their ownership
    #[payable]
    pub fn near_deposit_rewards(&mut self) {
        let total_reward_near = self.reward_tokens_all_time_count_near.0 + env::attached_deposit();
        self.reward_tokens_all_time_count_near = U128(total_reward_near);
        self.contract_rps_near = U128(total_reward_near / self.ft_functionality.ft_total_supply().0);
    }

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
                let total_reward_tokens = self.reward_tokens_all_time_count_token.0 + amount.0;
                self.reward_tokens_all_time_count_token = U128(total_reward_tokens);
                self.contract_rps_token =
                    U128(total_reward_tokens / self.ft_functionality.ft_total_supply().0);
                U128(0)
            }
            _ => panic!("Invalid msg param"),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::tests::*;

    #[rstest]
    /// Test near_deposit_rewards method
    /// ASSERT:
    /// (1) If deposited amount cannot be divided by total
    ///     supply of share tokens, don't immediatelly distribute
    #[case(REWARDS_TOKEN_ACCOUNT.parse().unwrap(), TOKEN_SUPPLY.0 - 1)]
    /// (2) Tokens get proportionally distributed between all holders
    #[case(REWARDS_TOKEN_ACCOUNT.parse().unwrap(), TOKEN_SUPPLY.0)]
    fn test_near_deposit_rewards(#[case] predecessor: AccountId, #[case] deposit_value: u128) {
        // setup
        let context = get_context(vec![], deposit_value, 0, predecessor, 0, Gas(200u64 * 10u64.pow(12)));
        testing_env!(context);
        let mut contract = init_contract(1);

        // call tested method
        contract.near_deposit_rewards();

        // perform assertions
        let mut rps_manager = contract
            .accounts_rps
            .get(&OWNER_ACCOUNT.parse::<AccountId>().unwrap())
            .unwrap();

        rps_manager.update_rps(contract.contract_rps_token.0, contract.contract_rps_near.0, TOKEN_SUPPLY.0);

        let rewards_balance = rps_manager.rewards_balance_near;

        if deposit_value >= TOKEN_SUPPLY.0 {
            assert_eq!(rewards_balance, TOKEN_SUPPLY);
        } else {
            assert_eq!(rewards_balance.0, 0);
        }
    }

    #[rstest]
    /// Test ft_on_transfer method
    /// ASSERT:
    /// (1) Method only accepts reward token as predecessor
    #[should_panic = "Invalid reward token, can only tranfer token:"]
    #[case(OWNER_ACCOUNT.parse().unwrap(), 10)]
    /// (2) If deposited amount cannot be divided by total
    ///     supply of share tokens, don't immediatelly distribute
    #[case(REWARDS_TOKEN_ACCOUNT.parse().unwrap(), TOKEN_SUPPLY.0 - 1)]
    /// (3) Tokens get proportionally distributed between all holders
    #[case(REWARDS_TOKEN_ACCOUNT.parse().unwrap(), TOKEN_SUPPLY.0)]
    fn test_ft_on_transfer(#[case] predecessor: AccountId, #[case] deposit_value: u128) {
        // setup
        let context = get_context(vec![], 0, 0, predecessor, 0, Gas(200u64 * 10u64.pow(12)));
        testing_env!(context);
        let mut contract = init_contract(1);

        // call tested method
        contract.ft_on_transfer(
            OWNER_ACCOUNT.parse().unwrap(),
            U128(deposit_value),
            "deposit_profits".to_string(),
        );

        // perform assertions
        let mut rps_manager = contract
            .accounts_rps
            .get(&OWNER_ACCOUNT.parse::<AccountId>().unwrap())
            .unwrap();

        rps_manager.update_rps(contract.contract_rps_token.0, contract.contract_rps_near.0, TOKEN_SUPPLY.0);

        let rewards_balance = rps_manager.rewards_balance_token;

        if deposit_value >= TOKEN_SUPPLY.0 {
            assert_eq!(rewards_balance, TOKEN_SUPPLY);
        } else {
            assert_eq!(rewards_balance.0, 0);
        }
    }

}
