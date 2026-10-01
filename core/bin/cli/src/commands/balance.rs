use chain_providers::ChainProviders;
use clap::Args;
use primitives::Chain;
use std::error::Error;

#[derive(Args)]
pub struct BalanceCommand {
    chain: Chain,
    address: String,
}

impl BalanceCommand {
    pub async fn run(&self, providers: &ChainProviders) -> Result<(), Box<dyn Error + Send + Sync>> {
        let chain = self.chain;
        let address = &self.address;

        match providers.get_balance_coin(chain, address.clone()).await {
            Ok(balance) => println!("{}: {}", balance.asset_id, balance.balance.available),
            Err(error) => eprintln!("Coin balance error: {}", error),
        }

        match providers.get_balance_assets(chain, address.clone()).await {
            Ok(balances) => {
                for balance in balances {
                    println!("{}: {}", balance.asset_id, balance.balance.available);
                }
            }
            Err(error) => eprintln!("Assets balance error: {}", error),
        }

        match providers.get_balance_staking(chain, address.clone()).await {
            Ok(Some(balance)) => println!("{} (staked): {}", balance.asset_id, balance.balance.staked),
            Ok(None) => {}
            Err(error) => eprintln!("Staking balance error: {}", error),
        }

        Ok(())
    }
}
