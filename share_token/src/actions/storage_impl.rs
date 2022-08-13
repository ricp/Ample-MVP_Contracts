use crate::*;

use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};
use near_sdk::Promise;

/// Implements users storage management for the pool.
#[near_bindgen]
impl StorageManagement for Contract {
    /// only takes registration only, no option to give more tokens
    #[allow(unused_variables)]
    #[payable]
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        let amount = env::attached_deposit();
        let account_id = account_id
            .map(|a| a.into())
            .unwrap_or_else(|| env::predecessor_account_id());
        let min_balance = self.storage_balance_bounds().min.0;
        let already_registered = self.accounts_rps.contains_key(&account_id);
        if amount < min_balance {
            panic!("{}", ERR_001_MIN_AMT);
        }

        // Registration only setups the account but doesn't leave space for tokens.
        if already_registered {
            if amount > 0 {
                Promise::new(env::predecessor_account_id()).transfer(amount);
            }
        } else {
            self.ft_functionality.internal_register_account(&account_id);
            self.update_user_rps(&account_id);
            let refund = amount - min_balance;
            if refund > 0 {
                Promise::new(env::predecessor_account_id()).transfer(refund);
            }
        }
        self.storage_balance_of(account_id).unwrap()
    }

    #[payable]
    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance {
        assert_one_yocto();
        let predecessor_account_id = env::predecessor_account_id();
        if let Some(storage_balance) = self.storage_balance_of(predecessor_account_id.clone()) {
            match amount {
                Some(amount) if amount.0 > 0 => {
                    env::panic_str("The amount is greater than the available storage balance");
                }
                _ => storage_balance,
            }
        } else {
            env::panic_str(
                format!("The account {} is not registered", &predecessor_account_id).as_str(),
            );
        }
    }

    #[payable]
    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        let force = force.unwrap_or(false);
        assert!(!force, "force option not available");

        let rewards = self.accounts_rps.get(&account_id);

        match rewards {
            None => return false,
            _ => (),
        }

        let rewards = rewards.unwrap();

        assert_eq!(
            rewards.rewards_balance,
            U128(0),
            "Account still has rewards to withdraw"
        );

        assert_eq!(
            self.ft_functionality.ft_balance_of(account_id.clone()),
            U128(0),
            "Account still has rewards tokens"
        );

        self.ft_functionality.accounts.remove(&account_id);
        self.accounts_rps.remove(&account_id);
        Promise::new(account_id.clone()).transfer(self.storage_balance_bounds().min.0 + 1);
        true
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        let required_storage_balance =
            Balance::from(STORAGE_BYTES_PER_USER) * env::storage_byte_cost();
        StorageBalanceBounds {
            min: required_storage_balance.into(),
            max: Some(required_storage_balance.into()),
        }
    }

    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
        if self.accounts_rps.contains_key(&account_id) {
            Some(StorageBalance {
                total: self.storage_balance_bounds().min,
                available: 0.into(),
            })
        } else {
            None
        }
    }
}
