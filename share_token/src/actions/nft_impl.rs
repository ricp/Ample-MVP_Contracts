//! Implementation of NEP-171 interface (nft standard)
//! 
//! Implements most methods with no functionality, since all
//! transfers happen through the NEP-141 interface.
//! 
//! View methods to show user NFTs personalize their metadata
//! according to the owner and the amount of shares that they own.

use std::vec;
use crate::*;

#[near_bindgen]
impl NonFungibleTokenCore for Contract {
    /// nft_transfer cannot be user, all transfer must happen
    /// throguh NEP-141 interface
    #[allow(unused_variables)]
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: String,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) {
    }

    /// nft_transfer_call cannot be user, all transfer must happen
    /// throguh NEP-141 interface
    #[allow(unused_variables)]
    fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: String,
        approval_id: Option<u64>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<bool> {
        PromiseOrValue::Value(false)
    }

    /// Displays data of the NFT representation of the artwork shares.
    /// Contract only supports nft_id == "0", other values return None.
    fn nft_token(&self, token_id: String) -> Option<Token> {
        let token_data = self.nft_instance_metadata.get().unwrap();
        match token_id.as_str() {
            "0" => Some(Token {
                token_id: "0".to_string(),
                owner_id: env::current_account_id(),
                metadata: Some(TokenMetadata {
                    title: Some(token_data.title),
                    description: Some(format!(
                        "{}. {}/{}",
                        token_data.description,
                        0,
                        self.ft_functionality.ft_total_supply().0
                    )),
                    media: Some(token_data.media),
                    media_hash: None,
                    copies: None,
                    issued_at: None,
                    expires_at: None,
                    starts_at: None,
                    updated_at: None,
                    extra: None,
                    reference: Some(token_data.reference),
                    reference_hash: None,
                }),
                approved_account_ids: None,
            }),
            _ => None,
        }
    }
}

#[near_bindgen]
impl NonFungibleTokenEnumeration for Contract {
    /// Method supposed to return the total quantity of NFTs
    /// in the contract. Always return 1 since the only NFT
    /// in the contract is a picture representing the artwork.
    fn nft_total_supply(&self) -> U128 {
        U128(1)
    }

    /// Displays data of the NFT representation of the artwork shares.
    /// Contract only supports nft_id == "0", other values return None.
    fn nft_tokens(
        &self,
        from_index: Option<U128>, // default: "0"
        limit: Option<u64>,       // default: unlimited (could fail due to gas limit)
    ) -> Vec<Token> {
        let token_data = self.nft_instance_metadata.get().unwrap();
        let token_instance = Token {
            token_id: "0".to_string(),
            owner_id: env::current_account_id(),
            metadata: Some(TokenMetadata {
                title: Some(token_data.title),
                description: Some(format!(
                    "{}. {}/{}",
                    token_data.description,
                    0,
                    self.ft_functionality.ft_total_supply().0
                )),
                media: Some(token_data.media),
                media_hash: None,
                copies: None,
                issued_at: None,
                expires_at: None,
                starts_at: None,
                updated_at: None,
                extra: None,
                reference: Some(token_data.reference),
                reference_hash: None,
            }),
            approved_account_ids: None,
        };
        match from_index {
            Some(index) => {
                let limit = limit.unwrap_or(1);
                if index.0 == 0 && limit > 0 {
                    vec![token_instance]
                } else {
                    vec![]
                }
            }
            None => {
                let limit = limit.unwrap_or(1);
                if limit > 0 {
                    vec![token_instance]
                } else {
                    vec![]
                }
            }
        }
    }

    fn nft_supply_for_owner(&self, account_id: AccountId) -> U128 {
        if self.ft_functionality.ft_balance_of(account_id).0 > 0 {
            U128(1)
        } else {
            U128(0)
        }
    }

    /// Displays data of the NFT representation of the artwork shares.
    /// Contract only supports from_index == "0" or None, 
    /// other values always return None.
    /// Only returns the NFT if the account_id owns shares. NFT is
    /// personalized to show the amount of shares that the account_id
    /// owns in this contract.
    fn nft_tokens_for_owner(
        &self,
        account_id: AccountId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<Token> {
        let token_data = self.nft_instance_metadata.get().unwrap();
        let has_nft = self.ft_functionality.ft_balance_of(account_id.clone()).0 > 0;
        let user_nft = Token {
            token_id: "0".to_string(),
            owner_id: env::current_account_id(),
            metadata: Some(TokenMetadata {
                title: Some(format!(
                    "{}. {}/{}",
                    token_data.title,
                    self.ft_functionality.ft_balance_of(account_id.clone()).0,
                    self.ft_functionality.ft_total_supply().0
                )),
                description: Some(format!(
                    "{}. {}/{}",
                    token_data.description,
                    self.ft_functionality.ft_balance_of(account_id).0,
                    self.ft_functionality.ft_total_supply().0
                )),
                media: Some(token_data.media),
                media_hash: None,
                copies: None,
                issued_at: None,
                expires_at: None,
                starts_at: None,
                updated_at: None,
                extra: None,
                reference: Some(token_data.reference),
                reference_hash: None,
            }),
            approved_account_ids: None,
        };
        if !has_nft {
            vec![]
        } else {
            match from_index {
                Some(index) => {
                    let limit = limit.unwrap_or(1);
                    if index.0 == 0 && limit > 0 {
                        vec![user_nft]
                    } else {
                        vec![]
                    }
                }
                None => {
                    let limit = limit.unwrap_or(1);
                    if limit > 0 {
                        vec![user_nft]
                    } else {
                        vec![]
                    }
                }
            }
        }
    }
}

/// Implements approval method for NEP-171. Only implements the interface
/// for compliance with standard, methods produce no effect, all transfers
/// must use NEP-141 interface
#[allow(unused_variables)]
#[near_bindgen]
impl NonFungibleTokenApproval for Contract {
    fn nft_approve(
        &mut self,
        token_id: TokenId,
        account_id: AccountId,
        msg: Option<String>,
    ) -> Option<Promise> {
        None
    }

    fn nft_revoke(&mut self, token_id: TokenId, account_id: AccountId) {}

    fn nft_revoke_all(&mut self, token_id: TokenId) {}

    fn nft_is_approved(
        &self,
        token_id: TokenId,
        approved_account_id: AccountId,
        approval_id: Option<u64>,
    ) -> bool {
        false
    }
}

/// Returns the nft_metadata of the contract. Method necessary
/// for NEAR wallet display of the NFT representation of shares.
#[near_bindgen]
impl NonFungibleTokenMetadataProvider for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.nft_contract_metadata.get().unwrap()
    }
}
