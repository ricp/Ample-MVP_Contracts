#[cfg(test)]
mod tests {

  use crate::*;

  /// Integration tests
  /// aims to test full aplication flow for x_token
  /// 0. Initialize accounts
  /// 1. Initialize contracts - share_token and reward_token
  /// 2. Users get reward_token
  /// 3. Users get share_token
  /// 4. Distribute rewards
  /// 5. Withdraw rewards
  #[tokio::test]
  async fn test_normal_flow() -> anyhow::Result<()> {
    let worker: Worker<Sandbox> = workspaces::sandbox().await?;

    let root = worker.root_account();

    // 0. Initialize accounts
    // CREATE USER ACCOUNTS
    let owner = create_user_account(&root, &worker, "owner").await;
    let user = create_user_account(&root, &worker, "user").await;
    let user2 = create_user_account(&root, &worker, "user2").await;

    // 1. Initialize contracts
    // DEPLOY REWARD_TOKEN
    let ft_wasm = get_wasm("token_contract.wasm")?;
    let ft_token = deploy_contract(&root, &worker, "ft_contract_reward", &ft_wasm).await;
    initialize_ft_contract(&worker, &ft_token, &owner).await;
    
    // DEPLOY SHARE_TOKEN
    let share_wasm = get_wasm("share_nft_token.wasm")?;
    let share_token = deploy_contract(&root, &worker, "share_nft_token", &share_wasm).await;

    let share_token_supply: u128 = 10_000;

    owner
      .call(&worker, share_token.id(), "new")
      .args_json(json!({
          "owner_id": owner.id(),
          "total_supply": share_token_supply.to_string(),
          "reward_token": ft_token.id(),
          "token_name": "name".to_string(),
          "token_symbol": "name".to_string(),
          "token_icon": "name".to_string(),
          "token_reference": "name".to_string(),
          "nft_instance_name": "name".to_string(),
          "nft_instance_description": "name".to_string(),
          "nft_instance_media_url": "name".to_string(),
      }))?
      .transact()
      .await?;

    let accounts = vec![
      &owner,
      &user,
      &user2,
      ft_token.as_account(),
      share_token.as_account(),
    ];
    let contracts = vec![&ft_token, &share_token];
    bulk_register_storage(&worker, accounts, contracts).await?;
    
    // 2. Users get reward_token
    // 3. Users get share_token

    let user_ft_balance: u128 = 1_000_000;
    let user_share_balance: u128 = 1_000;
    let owner_share_balance = share_token_supply - 2 * user_share_balance;

    ft_transfer(&worker, &owner, &ft_token, &user, user_ft_balance).await?;
    ft_transfer(&worker, &owner, &ft_token, &user2, user_ft_balance).await?;
    ft_transfer(&worker, &owner, &share_token, &user, user_share_balance).await?;
    ft_transfer(&worker, &owner, &share_token, &user2, user_share_balance).await?;

    // 4. Distribute rewards

    ft_transfer_call(&worker, &user, &ft_token, share_token.as_account(), user_ft_balance, "deposit_profits".to_string()).await?;

    let user1_profits = view_claimable_rewards(&worker, &share_token, &user).await?.parse::<u128>().unwrap();
    let user2_profits = view_claimable_rewards(&worker, &share_token, &user2).await?.parse::<u128>().unwrap();
    let owner_profits = view_claimable_rewards(&worker, &share_token, &owner).await?.parse::<u128>().unwrap();

    assert_eq!(user1_profits, (user_ft_balance * user_share_balance ) / share_token_supply);
    assert_eq!(user2_profits, (user_ft_balance * user_share_balance ) / share_token_supply);
    assert_eq!(owner_profits, (user_ft_balance * owner_share_balance ) / share_token_supply);

    ft_transfer_call(&worker, &user2, &ft_token, share_token.as_account(), user_ft_balance, "deposit_profits".to_string()).await?;

    let user1_profits = view_claimable_rewards(&worker, &share_token, &user).await?.parse::<u128>().unwrap();
    let user2_profits = view_claimable_rewards(&worker, &share_token, &user2).await?.parse::<u128>().unwrap();
    let owner_profits = view_claimable_rewards(&worker, &share_token, &owner).await?.parse::<u128>().unwrap();

    assert_eq!(user1_profits, (user_ft_balance * 2 * user_share_balance ) / share_token_supply);
    assert_eq!(user2_profits, (user_ft_balance * 2 * user_share_balance ) / share_token_supply);
    assert_eq!(owner_profits, (user_ft_balance * 2 * owner_share_balance ) / share_token_supply);

    // 5. Withdraw rewards
    let owner_balance_init = ft_balance_of(&worker, &ft_token, &owner).await?.parse::<u128>().unwrap();
    let user1_balance_init = ft_balance_of(&worker, &ft_token, &user).await?.parse::<u128>().unwrap();
    let user2_balance_init = ft_balance_of(&worker, &ft_token, &user2).await?.parse::<u128>().unwrap();

    claim_rewards(&worker, &owner, &share_token).await?;
    claim_rewards(&worker, &user, &share_token).await?;
    claim_rewards(&worker, &user2, &share_token).await?;
    
    let owner_balance_end = ft_balance_of(&worker, &ft_token, &owner).await?.parse::<u128>().unwrap();
    let user1_balance_end = ft_balance_of(&worker, &ft_token, &user).await?.parse::<u128>().unwrap();
    let user2_balance_end = ft_balance_of(&worker, &ft_token, &user2).await?.parse::<u128>().unwrap();
      
    assert_eq!(owner_balance_init + owner_profits, owner_balance_end);
    assert_eq!(user1_balance_init + user1_profits, user1_balance_end);
    assert_eq!(user2_balance_init + user2_profits, user2_balance_end);

    anyhow::Ok(())
  }
}
