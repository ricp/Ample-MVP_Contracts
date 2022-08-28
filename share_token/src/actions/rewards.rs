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
        assert_one_yocto();
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

#[cfg(test)]
mod tests {

    use near_sdk::PromiseResult;

    use super::*;
    use crate::tests::*;

    #[rstest]
    /// Test claim_rewards method
    /// ASSERT:
    /// (1) Call requires 1 yocto
    #[should_panic(expected = "Requires attached deposit of exactly 1 yoctoNEAR")]
    #[case(0, 0)]
    /// (2) Changes caller internal reward balance to 0
    /// (3) Emits promise with callback in case there are
    ///     rewards to withdraw
    #[case(1, 10)]
    fn test_claim_rewards(#[case] deposit: u128, #[case] internal_balance: u128) {
        // setup
        let context = get_context(
            vec![],
            deposit,
            0,
            USER_ACCOUNT.parse().unwrap(),
            0,
            Gas(300u64 * 10u64.pow(12)),
        );
        testing_env!(context);
        let user = USER_ACCOUNT.parse().unwrap();
        let mut contract = init_contract(1);
        register_user(&mut contract, &user, 100, internal_balance);

        // call tested method
        contract.claim_rewards();

        // perform assertions
        assert_eq!(
            contract.accounts_rps.get(&user).unwrap().rewards_balance.0,
            0
        );

        if internal_balance > 0 {
            let receipts = get_created_receipts();
            assert_eq!(receipts.len(), 2);

            assert_eq!(receipts[0].receiver_id, contract.reward_token.clone());
            assert_eq!(receipts[0].actions.len(), 1);

            if let VmAction::FunctionCall {
                function_name,
                args,
                gas: _,
                deposit,
            } = receipts[0].actions[0].clone()
            {
                assert_eq!(function_name, "ft_transfer");
                assert_eq!(deposit, 1);
                let json_args: serde_json::Value =
                    serde_json::from_str(from_utf8(&args).unwrap()).unwrap();
                assert_eq!(json_args["receiver_id"], user.to_string());
                assert_eq!(json_args["amount"], internal_balance.to_string());
            } else {
                panic!()
            };

            assert_eq!(receipts[1].receiver_id, CONTRACT_ACCOUNT.parse().unwrap());
            assert_eq!(receipts[1].actions.len(), 1);

            if let VmAction::FunctionCall {
                function_name,
                args,
                gas: _,
                deposit,
            } = receipts[1].actions[0].clone()
            {
                assert_eq!(function_name, "resolve_reward_transfer");
                assert_eq!(deposit, 0);
                let json_args: serde_json::Value =
                    serde_json::from_str(from_utf8(&args).unwrap()).unwrap();
                assert_eq!(json_args["receiver_id"], user.to_string());
                assert_eq!(json_args["amount"], internal_balance.to_string());
            } else {
                panic!()
            };
        }
    }

    #[rstest]
    /// Test resolve_reward_transfer method
    /// ASSERT:
    /// (1) Method only allows contract to call it
    /// ***Assertion can't currently be made because of
    ///    flaw in #[private] macro in mocked context
    // #[should_panic(expected = "Method resolve_reward_transfer is private")]
    // #[case(OWNER_ACCOUNT.parse().unwrap(), false, 100)]
    /// (2) If promise succeeded does nothing
    #[case(CONTRACT_ACCOUNT.parse().unwrap(), true, 100)]
    /// (3) If promise fails resume user balance
    #[case(CONTRACT_ACCOUNT.parse().unwrap(), false, 100)]
    fn test_resolve_reward_transfer(
        #[case] caller: AccountId,
        #[case] promise_success: bool,
        #[case] transferred_balance: u128,
    ) {
        // setup
        let context = get_context(vec![], 0, 0, caller, 0, Gas(50u64 * 10u64.pow(12)));
        
        let promise_result = if promise_success {
            PromiseResult::Successful(vec![])
        } else {
            PromiseResult::Failed
        };
        
        testing_env!(
            context,
            VMConfig::test(),
            RuntimeFeesConfig::test(),
            HashMap::default(),
            vec![promise_result]
          );
        let user = USER_ACCOUNT.parse().unwrap();
        let mut contract = init_contract(1);
        register_user(&mut contract, &user, 100, 0);

        // call tested method
        contract.resolve_reward_transfer(user.clone(), U128(transferred_balance));
    
        // make assertions
        if promise_success {
            assert_eq!(
                contract.accounts_rps.get(&user).unwrap().rewards_balance.0,
                0
            );
        } else {
            assert_eq!(
                contract.accounts_rps.get(&user).unwrap().rewards_balance.0,
                transferred_balance
            );
        }
    }
}
