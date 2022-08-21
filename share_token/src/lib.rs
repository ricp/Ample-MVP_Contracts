use near_contract_standards::fungible_token::core::FungibleTokenCore;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::json_types::U128;
#[allow(unused_imports)]
use near_sdk::serde::{self, Serialize, Deserialize};
use near_sdk::{
    env, near_bindgen, utils::assert_one_yocto, AccountId, Balance, BorshStorageKey,
    PanicOnDefault, PromiseOrValue, Promise
};

use near_contract_standards;
use near_contract_standards::fungible_token::events::{FtBurn, FtMint};
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider,
};
use near_contract_standards::fungible_token::resolver::FungibleTokenResolver;
use near_contract_standards::fungible_token::FungibleToken;
use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata,
};
use near_contract_standards::non_fungible_token::core::NonFungibleTokenCore;
use near_contract_standards::non_fungible_token::enumeration::NonFungibleTokenEnumeration;
use near_contract_standards::non_fungible_token::approval::NonFungibleTokenApproval;
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_contract_standards::non_fungible_token::events::{NftMint, NftBurn};

mod actions;
mod errors;
mod rps_manager;

use errors::*;
use rps_manager::RpsManager;

const STORAGE_BYTES_PER_USER: u64 = 0;

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct Contract {
    pub ft_functionality: FungibleToken,
    pub reward_token: AccountId,
    pub reward_token_balance: U128,
    pub contract_rps: U128,
    pub accounts_rps: LookupMap<AccountId, RpsManager>,
    pub token_metadata: LazyOption<FungibleTokenMetadata>,
    pub nft_contract_metadata: LazyOption<NFTContractMetadata>,
    pub nft_instance_metadata: LazyOption<NftInstanceData>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde( crate = "near_sdk::serde" )]
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
    #[init]
    pub fn new(
        owner_id: AccountId,
        total_supply: U128,
        reward_token: AccountId,
        token_metadata: FungibleTokenMetadata,
        nft_contract_metadata: NFTContractMetadata,
        nft_instance_metadata: NftInstanceData,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        let mut this = Self {
            ft_functionality: FungibleToken::new(StorageKey::FungibleToken),
            reward_token,
            reward_token_balance: U128(0),
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
            memo: Some("received first shares")
        }.emit();

        this
    }
}

// Implement relevant internal methods
impl Contract {
    pub fn update_user_rps(&mut self, account_id: &AccountId) {
        let mut user_rps = self
            .accounts_rps
            .get(account_id)
            .unwrap_or(RpsManager::new(self.contract_rps.0));

        let user_balance = self.ft_functionality.ft_balance_of(account_id.clone());

        user_rps.update_rps(self.contract_rps.0, user_balance.0);

        self.accounts_rps.insert(account_id, &user_rps);
    }

    pub fn emit_sender_nft_events(&self, account_id: &AccountId) {
        if self.ft_functionality.ft_balance_of(account_id.clone()) == U128(0) {
            NftBurn {
                owner_id: account_id,
                token_ids: &["0"],
                authorized_id: None,
                memo: Some("transferred all shares")
            }.emit()
        }
    }

    pub fn emit_receiver_nft_events(&self, account_id: &AccountId) {
        if self.ft_functionality.ft_balance_of(account_id.clone()) == U128(0) {
            NftMint {
                owner_id: account_id,
                token_ids: &["0"],
                memo: Some("received first shares")
            }.emit()
        }
    }

}

//implement necessary methods for standard implementation
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
