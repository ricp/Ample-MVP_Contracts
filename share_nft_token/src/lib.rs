//! Ample Protocol - Artwork Shares Contract
//!
//! This contract is a NEP-141 compliant token representing shares of
//! ownership of a piece of intellectual property.
//!
//! Each token is an ideal fraction of ownership. The contract supports
//! the distribution of dividends to all shares of ownership.
//! Upon initialization of the contract, the deployer must choose a
//! NEP-141 token in which all dividends are going to be paid.
//!
//! Everytime the chosen token is transfered to this contract it gets automatically
//! transferred to owners of shares in the proportion of their ownership
//! using the [scalable rewar distribution algorithm](http://batog.info/papers/scalable-reward-distribution.pdf)
//!
//! Besides the NEP-141 functionality, the contract also implements
//! the NEP-171 standard. This means the shares show up in the NEAR
//! wallet in the collectibles section, displaying the user's shares
//! as a NFT.

use near_contract_standards::fungible_token::core::FungibleTokenCore;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::json_types::U128;
#[allow(unused_imports)]
use near_sdk::serde::{self, Deserialize, Serialize};
use near_sdk::{
    env, log, near_bindgen, utils::assert_one_yocto, AccountId, Balance, BorshStorageKey, Gas,
    PanicOnDefault, Promise, PromiseOrValue,
};

use near_contract_standards;
use near_contract_standards::fungible_token::events::{FtBurn, FtMint};
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider,
};
use near_contract_standards::fungible_token::resolver::FungibleTokenResolver;
use near_contract_standards::fungible_token::FungibleToken;
use near_contract_standards::non_fungible_token::approval::NonFungibleTokenApproval;
use near_contract_standards::non_fungible_token::core::NonFungibleTokenCore;
use near_contract_standards::non_fungible_token::enumeration::NonFungibleTokenEnumeration;
use near_contract_standards::non_fungible_token::events::{NftBurn, NftMint};
use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata,
};
use near_contract_standards::non_fungible_token::{Token, TokenId};

mod actions;
mod ext_interface;
mod rps_manager;

use rps_manager::RpsManager;

/// Exact byte size of data stored for each user that registers
/// in the contract. Contract takes 154 bytes for information plus
/// 2 bytes for each UTF8 char in account ID (LookupMaps in
/// ft_functionality and accounts_rps), which amounts to
/// 282 bytes. Consider 300 to give a 10% wiggle room.
const STORAGE_BYTES_PER_USER: u64 = 300;

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct Contract {
    /// Stores entire nep-141 functionality that represents
    /// each share of the artwork
    pub ft_functionality: FungibleToken,
    /// The NEP-141 token type in which dividends from the
    /// artwork are paid. Is a reference to the account of
    /// the token contract.
    pub reward_token: AccountId,
    /// All time count of reward tokens received
    pub reward_tokens_all_time_count: U128,
    /// Revenue per share counter. part of the [scalable reward
    /// distribution algorithm](http://batog.info/papers/scalable-reward-distribution.pdf)
    pub contract_rps: U128,
    /// Revenue per share claimed by each individual account
    /// up to its latest interaction. part of the [scalable reward
    /// distribution algorithm](http://batog.info/papers/scalable-reward-distribution.pdf)
    pub accounts_rps: LookupMap<AccountId, RpsManager>,
    /// NEP-141 metadata for the shares
    pub token_metadata: LazyOption<FungibleTokenMetadata>,
    /// NEP-171 metadata for NFT representation of the shares.
    pub nft_contract_metadata: LazyOption<NFTContractMetadata>,
    /// Metadata for the artwork's NFT, includes name,
    /// image and other relevant data.
    pub nft_instance_metadata: LazyOption<NftInstanceData>,
}

/// NFT data to display for owners of shares
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct NftInstanceData {
    pub title: String,
    pub description: String,
    pub media: String,
    pub reference: String,
}

#[derive(BorshDeserialize, BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    FungibleToken,
    AccontRps,
    FungibleTokenMetadata,
    NonFungibleTokenMetadata,
    InstanceNonFungibleTokenMetadata,
}

#[allow(dead_code)]
#[near_bindgen]
impl Contract {
    /// Initializes the contract and sends entire initial balance
    /// to owner.
    #[init]
    pub fn new(
        owner_id: AccountId,
        total_supply: U128,
        reward_token: AccountId,
        token_name: String,
        token_symbol: String,
        token_icon: Option<String>,
        token_reference: String,
        nft_instance_name: String,
        nft_instance_description: String,
        nft_instance_media_url: String,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        let token_metadata = FungibleTokenMetadata {
            spec: "ft-1.0.0".to_string(),
            name: format!("{}-token", token_name.clone()),
            symbol: token_symbol.clone(),
            icon: token_icon.clone(),
            reference: Some(token_reference.clone()),
            reference_hash: None,
            decimals: 0,
        };
        let nft_contract_metadata = NFTContractMetadata {
            spec: "nft-1.0.0".to_string(),
            name: format!("{}-nft", token_name.clone()),
            symbol: token_symbol,
            icon: token_icon,
            base_uri: None,
            reference: Some(token_reference.clone()),
            reference_hash: None,
        };
        let nft_instance_metadata = NftInstanceData {
            title: nft_instance_name,
            description: nft_instance_description,
            media: nft_instance_media_url,
            reference: token_reference,
        };
        let mut this = Self {
            ft_functionality: FungibleToken::new(StorageKey::FungibleToken),
            reward_token,
            reward_tokens_all_time_count: U128(0),
            contract_rps: U128(0),
            accounts_rps: LookupMap::new(StorageKey::AccontRps),
            token_metadata: LazyOption::new(
                StorageKey::FungibleTokenMetadata,
                Some(&token_metadata),
            ),
            nft_contract_metadata: LazyOption::new(
                StorageKey::NonFungibleTokenMetadata,
                Some(&nft_contract_metadata),
            ),
            nft_instance_metadata: LazyOption::new(
                StorageKey::InstanceNonFungibleTokenMetadata,
                Some(&nft_instance_metadata),
            ),
        };
        this.update_user_rps(&owner_id);
        this.ft_functionality.internal_register_account(&owner_id);
        this.ft_functionality
            .internal_deposit(&owner_id, total_supply.into());
        FtMint {
            owner_id: &owner_id,
            amount: &total_supply,
            memo: Some("Single Mint event on creation"),
        }
        .emit();
        NftMint {
            owner_id: &owner_id,
            token_ids: &["0"],
            memo: Some("received first shares"),
        }
        .emit();

        this
    }
}

/// Implements relevant internal methods for reward distribution
/// bookkeeping and NFT representation of shares display
impl Contract {
    /// Compares current contract_rps and user's account_rps
    /// if user has rewards to receive, credit them to user's
    /// RpsManager and update's account_rps to contract_rps' value
    pub fn update_user_rps(&mut self, account_id: &AccountId) {
        let mut user_rps = self
            .accounts_rps
            .get(account_id)
            .unwrap_or(RpsManager::new(self.contract_rps.0));

        let user_balance = self.ft_functionality.ft_balance_of(account_id.clone());

        user_rps.update_rps(self.contract_rps.0, user_balance.0);

        self.accounts_rps.insert(account_id, &user_rps);
    }

    /// Updates user's rewards balance with current contract_rps and then
    /// zeroes it, returns total amount of rewards that must be transferred
    /// to user.
    pub fn withdraw_rewards(&mut self, account_id: &AccountId) -> U128 {
        let mut user_rps = self
            .accounts_rps
            .get(account_id)
            .unwrap_or(RpsManager::new(self.contract_rps.0));

        let user_balance = self.ft_functionality.ft_balance_of(account_id.clone());
        user_rps.update_rps(self.contract_rps.0, user_balance.0);

        let reward_count = user_rps.withdraw_rewards();
        self.accounts_rps.insert(account_id, &user_rps);
        reward_count
    }

    /// Rolls back effects from withdraw_rewards. Is called in case the
    /// token transfer fails and the user's internal balance must be
    /// reconstituted.
    pub fn rollback_withdraw_reward(&mut self, account_id: &AccountId, amount: u128) {
        let mut user_rps = self
            .accounts_rps
            .get(account_id)
            .unwrap_or(RpsManager::new(self.contract_rps.0));
        user_rps.rewards_balance = U128(user_rps.rewards_balance.0 + amount);
        self.accounts_rps.insert(account_id, &user_rps);
    }

    /// This method must be called every time a user transfers shares.
    /// If user has no more shares, emits an event representing the burn
    /// of their NFT
    pub fn emit_sender_nft_events(&self, account_id: &AccountId) {
        if self.ft_functionality.ft_balance_of(account_id.clone()) == U128(0) {
            NftBurn {
                owner_id: account_id,
                token_ids: &["0"],
                authorized_id: None,
                memo: Some("transferred all shares"),
            }
            .emit()
        }
    }

    /// This method must be called every time a user receives shares.
    /// If user previously had no shares, emits an event representing
    /// the mint of the NFT representation of shares
    pub fn emit_receiver_nft_events(&self, account_id: &AccountId) {
        if self.ft_functionality.ft_balance_of(account_id.clone()) == U128(0) {
            NftMint {
                owner_id: account_id,
                token_ids: &["0"],
                memo: Some("received first shares"),
            }
            .emit()
        }
    }
}

impl Contract {
    fn on_tokens_burned(&mut self, account_id: AccountId, amount: u128) {
        FtBurn {
            owner_id: &account_id,
            amount: &U128(amount),
            memo: None,
        }
        .emit();
    }
}

#[cfg(test)]
mod tests {
    pub use near_sdk::collections::LazyOption;
    pub use near_sdk::mock::VmAction;
    pub use near_sdk::serde_json::{self, json};
    pub use near_sdk::test_utils::{get_created_receipts, get_logs};
    pub use near_sdk::{testing_env, Balance, Gas, MockedBlockchain, VMContext};
    pub use near_sdk::{RuntimeFeesConfig, VMConfig, PromiseResult};

    pub use rstest::{fixture, rstest};

    pub use std::collections::HashMap;
    pub use std::convert::{TryFrom, TryInto};
    pub use std::panic::{catch_unwind, UnwindSafe};
    pub use std::str::from_utf8;

    pub use super::*;

    /// Mocked contract account id
    pub const CONTRACT_ACCOUNT: &str = "contract.testnet";
    /// Mocked rewards token account id
    pub const REWARDS_TOKEN_ACCOUNT: &str = "rewards.testnet";
    /// Mocked owner account id
    pub const OWNER_ACCOUNT: &str = "owner.testnet";
    /// Mocked regular user account id
    pub const USER_ACCOUNT: &str = "user.testnet";
    /// Total token supply to use in tests
    pub const TOKEN_SUPPLY: U128 = U128(100000);

    /// Initializes mocked blockchain context
    pub fn get_context(
        input: Vec<u8>,
        attached_deposit: u128,
        account_balance: u128,
        signer_id: AccountId,
        block_timestamp: u64,
        prepaid_gas: Gas,
    ) -> VMContext {
        VMContext {
            current_account_id: CONTRACT_ACCOUNT.parse().unwrap(),
            signer_account_id: signer_id.clone(),
            signer_account_pk: vec![0; 33].try_into().unwrap(),
            predecessor_account_id: signer_id.clone(),
            input,
            block_index: 0,
            block_timestamp,
            account_balance,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit,
            prepaid_gas,
            random_seed: [0; 32],
            view_config: None,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    /// Initializes contract with random seed as storage keys to
    /// guarantee no collisions
    pub fn init_contract(seed: u128) -> Contract {
        let hash1 = env::keccak256(&seed.to_be_bytes());
        let hash2 = env::keccak256(&hash1[..]);
        let hash3 = env::keccak256(&hash2[..]);
        let hash4 = env::keccak256(&hash3[..]);
        let hash5 = env::keccak256(&hash4[..]);
        let token_metadata = FungibleTokenMetadata {
            spec: "ft-1.0.0".to_string(),
            name: format!("{}-token", "token_name"),
            symbol: "token_symbol".to_string(),
            icon: Some("token_icon".to_string()),
            reference: Some("token_reference".to_string()),
            reference_hash: None,
            decimals: 0,
        };
        let nft_contract_metadata = NFTContractMetadata {
            spec: "nft-1.0.0".to_string(),
            name: format!("{}-token", "token_name"),
            symbol: "token_symbol".to_string(),
            icon: Some("token_icon".to_string()),
            base_uri: None,
            reference: Some("token_reference".to_string()),
            reference_hash: None,
        };
        let nft_instance_metadata = NftInstanceData {
            title: "nft_instance_name".to_string(),
            description: "nft_instance_description".to_string(),
            media: "nft_instance_media_url".to_string(),
            reference: "token_reference".to_string(),
        };
        let mut this = Contract {
            ft_functionality: FungibleToken::new(hash1),
            reward_token: REWARDS_TOKEN_ACCOUNT.parse().unwrap(),
            reward_tokens_all_time_count: U128(0),
            contract_rps: U128(0),
            accounts_rps: LookupMap::new(hash2),
            token_metadata: LazyOption::new(hash3, Some(&token_metadata)),
            nft_contract_metadata: LazyOption::new(hash4, Some(&nft_contract_metadata)),
            nft_instance_metadata: LazyOption::new(hash5, Some(&nft_instance_metadata)),
        };
        this.update_user_rps(&OWNER_ACCOUNT.parse().unwrap());
        this.ft_functionality
            .internal_register_account(&OWNER_ACCOUNT.parse().unwrap());
        this.ft_functionality
            .internal_deposit(&OWNER_ACCOUNT.parse().unwrap(), TOKEN_SUPPLY.into());
        this
    }

    pub fn register_user(
        contract: &mut Contract,
        user: &AccountId,
        token_balance: u128,
        rewards_balance: u128,
    ) {
        contract.update_user_rps(user);
        contract.ft_functionality.internal_register_account(user);
        contract
            .ft_functionality
            .internal_deposit(user, token_balance);
        let mut internal_rps = contract.accounts_rps.get(user).unwrap();
        internal_rps.rewards_balance = U128(rewards_balance);
        contract.accounts_rps.insert(user, &internal_rps);
    }

    #[rstest]
    fn test_new() {
        let context = get_context(
            vec![],
            0,
            0,
            OWNER_ACCOUNT.parse().unwrap(),
            0,
            Gas(300u64 * 10u64.pow(12)),
        );
        testing_env!(context);

        let contract = Contract::new(
            OWNER_ACCOUNT.parse().unwrap(),
            TOKEN_SUPPLY,
            REWARDS_TOKEN_ACCOUNT.parse().unwrap(),
            "NAME_HERE".to_string(),
            "SYMBOL_HERE".to_string(),
            Some("ICON_HERE".to_string()),
            "REFERENCE_HERE".to_string(),
            "NFT_NAME_HERE".to_string(),
            "NFT_DESC_HERE".to_string(),
            "NFT_MEDIA_HERE".to_string(),
        );

        assert_eq!(
            TOKEN_SUPPLY,
            contract
                .ft_functionality
                .ft_balance_of(OWNER_ACCOUNT.parse().unwrap())
        );
    }
}
