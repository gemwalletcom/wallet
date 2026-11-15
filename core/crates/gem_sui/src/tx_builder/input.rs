use crate::types::ObjectInput;
#[cfg(feature = "rpc")]
use crate::{
    SuiClient, SuiError,
    models::{Coin, OwnedCoins},
};
#[cfg(feature = "rpc")]
use futures::try_join;

#[derive(Clone)]
pub struct TransactionBuilderInput {
    pub sender: String,
    pub gas_price: u64,
    pub gas_budget: u64,
    pub gas_objects: Vec<ObjectInput>,
    pub address_balance: u64,
}

impl TransactionBuilderInput {
    pub fn new(sender: impl Into<String>, gas_price: u64, gas_budget: u64, gas_objects: Vec<ObjectInput>) -> Self {
        Self {
            sender: sender.into(),
            gas_price,
            gas_budget,
            gas_objects,
            address_balance: 0,
        }
    }

    pub fn with_gas_budget(&self, gas_budget: u64) -> Self {
        let mut input = self.clone();
        input.gas_budget = gas_budget;
        input
    }

    #[cfg(feature = "rpc")]
    pub async fn prefetch(client: &SuiClient, sender: &str, gas_budget: u64) -> Result<Self, SuiError> {
        let gas_price = async { client.get_gas_price().await.map_err(SuiError::from_display) };
        let gas_coins = async { client.get_gas_coins(sender).await.map_err(SuiError::from_display) };
        let (gas_price, gas_coins) = try_join!(gas_price, gas_coins)?;
        Self::from_gas_coins(sender, gas_price, gas_budget, &gas_coins)
    }

    #[cfg(feature = "rpc")]
    fn from_gas_coins(sender: &str, gas_price: u64, gas_budget: u64, gas_coins: &OwnedCoins<Coin>) -> Result<Self, SuiError> {
        if gas_coins.coins.is_empty() {
            return Err(SuiError::NoGasCoins);
        }
        let gas_objects = gas_coins.coins.iter().map(Coin::to_input).collect();

        Ok(Self {
            address_balance: gas_coins.address_balance,
            ..Self::new(sender, gas_price, gas_budget, gas_objects)
        })
    }
}

#[cfg(all(test, feature = "rpc"))]
mod tests {
    use super::*;
    use crate::SUI_COIN_TYPE;

    #[test]
    fn test_from_gas_coins() {
        let sender = "0xa9bd0493f9bd1f792a4aedc1f99d54535a75a46c38fd56a8f2c6b7c8d75817a1";
        let coins = OwnedCoins::new(SUI_COIN_TYPE.to_string(), vec![Coin::mock_sui()], 626_400_000_000);
        let input = TransactionBuilderInput::from_gas_coins(sender, 750, 50_000_000, &coins).unwrap();

        assert_eq!(input.gas_objects, vec![Coin::mock_sui().to_input()]);
        assert_eq!(input.address_balance, 626_400_000_000);

        let coins_only = OwnedCoins::new(SUI_COIN_TYPE.to_string(), vec![Coin::mock_sui()], 0);
        assert_eq!(TransactionBuilderInput::from_gas_coins(sender, 750, 50_000_000, &coins_only).unwrap().address_balance, 0);

        assert!(TransactionBuilderInput::from_gas_coins(sender, 750, 50_000_000, &OwnedCoins::default()).is_err());
    }
}
