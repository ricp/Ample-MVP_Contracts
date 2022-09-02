# Ample share token contract
# Usage manual

This contract is implemented as a NEP-141 (fungible token standard) compliant token that distributes
dividends to all its holders in the proportion of their ownership.  
Besides that, the contract also implements NEP-171 (NFT standard) so that the ownership of share tokens
also shows up as an NFT in the owner's NEAR wallet.  
Rewards can be distributed both in NEAR and a selected NEP-141 token. 

## Deployment

### Requirements
To deploy the contract you must install node.js and the near cli.
- Node.js: [Installation Instructions](https://nodejs.org/en/)
- near-cli: [Installation Instructions](https://docs.near.org/tools/near-cli)

To deploy the application on mainnet you need to set an environment variable:
```
export NEAR_ENV=mainnet
```
The setup is default mainnet, but you can also change it back to testnet.

You also need a compatible rust setup as described in the near-sdk library requirements [described here](https://github.com/near/near-sdk-rs).

### Compile the contract 
First deployment step is compiling the contract code. You need to run:
```
RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release
``` 

### Deploy and initialize
To initialize the contract you'll need to define the contract's setup parameters:
- deploy_account_id -> Account to which the contract is going to be deployed;
- owner_id -> Account that is going to receive all tokens upon initialization;
- total_supply -> Quantity of tokens that are going to be created;
- reward_token -> Address of the token that is going to be used to pay dividends to share holders;
- token_name -> Name that is going to be displayed on NEAR wallet for the token and NFT;
- token_symbol -> Ticker of the token and NFT to be displayed on NEAR wallet and helper applications;
- token_icon -> URL to token icon, used for both Token and NFT. Must be a data URL, it's recommended to use an optimized SVG as described [here](https://nomicon.io/Standards/Tokens/FungibleToken/Metadata);
- token_reference -> URL to token/NFT metadata. This should include all metadata regarding the intellectual property as well. The link should point to a decentralized storage such as Arweave or IPFS;
- nft_instance_name -> Name of the NFT "image" to be displayed in the NEAR wallet and other NFT applications. Will have (<user_shares>/<total_shares>) appended at the end.
- nft_instance_description -> Description of the NFT "image" to be displayed in the NEAR wallet and other NFT applications. Will have (<user_shares>/<total_shares>) appended at the end.
- nft_instance_media_url -> URL to NFT image, Should point to decentralized storage link such as Arweave or IPFS.

After the contract has been initialized there is no way to change these variables, caution is recommended when inputing these values.

Substitute all values between <> for the actual selected values:

```
near deploy --accountId <deploy_account_id> --wasmFile target/wasm32-unknown-unknown/release/share_nft_token.wasm --initFunction new --initArgs '{"owner_id": "<owner_id>", "total_supply": "<total_supply>", "reward_token": "<reward_token>", "token_name": "<token_name>", "token_symbol": "<token_symbol>", "token_icon": "<token_icon>", "token_reference": "<token_reference>", "nft_instance_name": "<nft_instance_name>", "nft_instance_description": "<nft_instance_description>", "nft_instance_media_url": "<nft_instance_media_url>"}'
```

### NEP-141 interface
After deployment the <total_supply> is going to be entirelly transferred to <owner_id>. To transfer tokens to other, utilize the NEP-141 interface, available [here](https://nomicon.io/Standards/Tokens/FungibleToken/Core).

### NEP-171 interface
The contract only implements the NEP-171 view methods, which are necessary for displaying the tokens to the owner as a NFT in their NEAR wallet and other web3 applications. All change methods available in NEP-171 produce no effect in this contract. The full NEP-171 interface is available [here](https://nomicon.io/Standards/Tokens/NonFungibleToken/)

### Distribute dividends
Any account can distribute dividends to all token holders by transferring the <reward_token> to this contract using the following CLI command:

- amount -> amount of tokens that you want to distribute as dividends
- depositor -> account that wants to pay for dividends being distributed

```
near call <reward_token> ft_transfer_call '{"receiver_id": "<deploy_account_id>", "amount": "<amount>", "msg": "deposit_profits"}' --accountId <depositor> --depositYocto 1
```

Any account can also distribute dividends to all token holders by transferring NEAR to this contract using the following CLI command:

```
near call <reward_token> near_deposit_rewards '{}' --accountId <depositor> --depositYocto <amount>
```

### Withdraw dividends
To check how much an account has received in dividends not yet withdrawn:

- user_account -> Account whose dividend balance you want to check

```
near view <deploy_account_id> view_claimable_rewards '{"account_id": "<user_account>"}'
```

To withdraw your received rewards:
```
near call <deploy_account_id> claim_rewards --accountId <user_account> --depositYocto 1
```

* Note that to withdraw tokens your account must be registered in the <reward_token> contract. For more information check out the NEP-141 [documentation](https://nomicon.io/Standards/Tokens/FungibleToken/Core)