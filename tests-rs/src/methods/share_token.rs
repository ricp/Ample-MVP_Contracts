use crate::*;

pub async fn claim_rewards(
  worker: &Worker<Sandbox>,
  sender: &Account,
  contract: &Contract,
  
) -> anyhow::Result<()> {
  sender
    .call(&worker, contract.id(), "claim_rewards")
    .args_json(json!({}))?
    .deposit(1)
    .gas(GAS_LIMIT)
    .transact()
    .await?;
  anyhow::Ok(())
}

pub async fn near_deposit_rewards(
  worker: &Worker<Sandbox>,
  sender: &Account,
  contract: &Contract,
  deposit: u128,
  
) -> anyhow::Result<()> {
  sender
    .call(&worker, contract.id(), "near_deposit_rewards")
    .args_json(json!({}))?
    .deposit(deposit)
    .gas(GAS_LIMIT)
    .transact()
    .await?;
  anyhow::Ok(())
}

pub async fn view_claimable_rewards(
  worker: &Worker<Sandbox>,
  contract: &Contract,
  account: &Account,
) -> anyhow::Result<HashMap<String, String>> {
  anyhow::Ok(
    contract
      .view(
        worker,
        "view_claimable_rewards",
        json!({
          "account_id": account.id()
        })
        .to_string()
        .into_bytes(),
      )
      .await?
      .json()?,
  )
}