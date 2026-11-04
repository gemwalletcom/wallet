use std::error::Error;

use async_trait::async_trait;
use chain_traits::ChainStaking;
use futures::{future::try_join_all, try_join};
use gem_client::Client;
use num_bigint::BigUint;
use primitives::{DelegationBase, DelegationValidator, asset_constants::TON_TSTON_TOKEN_ID};

use crate::{
    address::Address,
    constants::TONSTAKERS_POOL_ADDRESS,
    models::NftItem,
    provider::staking_mapper::{map_delegations, map_staking_apy, map_validator},
    rpc::client::TonClient,
};

#[async_trait]
impl<C: Client> ChainStaking for TonClient<C> {
    async fn get_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        let pool = self.get_pool_full_data(TONSTAKERS_POOL_ADDRESS).await?;
        Ok(Some(map_staking_apy(&pool)))
    }

    async fn get_staking_validators(&self, _apy: Option<f64>) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        let pool = self.get_pool_full_data(TONSTAKERS_POOL_ADDRESS).await?;
        Ok(vec![map_validator(&pool)])
    }

    async fn get_staking_delegations(&self, address: String) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        let (staked_shares, pool) = try_join!(self.get_jetton_balance(&address, TON_TSTON_TOKEN_ID), self.get_pool_full_data(TONSTAKERS_POOL_ADDRESS))?;
        let (deposit_bills, withdrawal_bills) = try_join!(self.get_bill_amounts(&address, pool.deposit_payout.as_ref()), self.get_bill_amounts(&address, pool.withdrawal_payout.as_ref()))?;
        Ok(map_delegations(staked_shares, &pool, &deposit_bills, &withdrawal_bills))
    }
}

impl<C: Client> TonClient<C> {
    async fn get_bill_amounts(&self, owner: &str, collection: Option<&Address>) -> Result<Vec<BigUint>, Box<dyn Error + Sync + Send>> {
        let Some(collection) = collection else {
            return Ok(vec![]);
        };
        let collection = collection.encode_raw();
        let bills = self.get_nft_items_in_collection(owner, &collection).await?;
        let amounts = try_join_all(bills.nft_items.iter().map(|bill| self.get_verified_bill_amount(&collection, bill))).await?;
        Ok(amounts.into_iter().flatten().collect())
    }

    async fn get_verified_bill_amount(&self, collection: &str, bill: &NftItem) -> Result<Option<BigUint>, Box<dyn Error + Sync + Send>> {
        let Some(index) = bill.index.as_deref() else {
            return Ok(None);
        };
        if self.get_nft_address_by_index(collection, index).await? != Address::parse(&bill.address)? {
            return Ok(None);
        }
        Ok(Some(self.get_bill_amount(&bill.address).await?))
    }
}

#[cfg(test)]
mod tests {
    use gem_client::testkit::MockClient;
    use primitives::DelegationState;

    use super::*;

    const TEST_OWNER: &str = "0:0CE8D22B5718169E80D0D3B9E344861D874E0C0D6F892B7966D8F29979A7636B";
    const TEST_BILL: &str = "0:7B990536CE680CBCC0199F015A236054CBA32D67D779A6FCFD8AC74CCC913D33";
    const TEST_TSTON_WALLET: &str = "EQAlgB03OjJKdXrlwZiGJD5snSzPKF2VL5bErJn_cqJANGH9";

    #[tokio::test]
    async fn test_get_staking_delegations_skips_a_bill_the_collection_did_not_issue() {
        let wallet_response = format!(r#"{{"exit_code":0,"stack":[{{"type":"cell","value":"{}"}}]}}"#, Address::parse(TEST_TSTON_WALLET).unwrap().to_boc_base64().unwrap());
        let bill_response = format!(r#"{{"exit_code":0,"stack":[{{"type":"cell","value":"{}"}}]}}"#, Address::parse(TEST_BILL).unwrap().to_boc_base64().unwrap());
        let client = TonClient::new(
            MockClient::new()
                .with_get(|path| match path {
                    "/api/v3/nft/items?owner_address=0:0CE8D22B5718169E80D0D3B9E344861D874E0C0D6F892B7966D8F29979A7636B&collection_address=0:2ca6ad3f72fffa324d825d318985164ba14e0c1daf4992ba1312976333a5b857&limit=1000&offset=0" => Ok(
                        br#"{"nft_items":[{"address":"0:7B990536CE680CBCC0199F015A236054CBA32D67D779A6FCFD8AC74CCC913D33","index":"0"},{"address":"0:1111111111111111111111111111111111111111111111111111111111111111","index":"1"}],"metadata":{}}"#.to_vec(),
                    ),
                    path => panic!("unexpected path {path}"),
                })
                .with_post(move |_, body| {
                    let request: serde_json::Value = serde_json::from_slice(body).unwrap();
                    match (request["address"].as_str().unwrap(), request["method"].as_str().unwrap()) {
                        (TON_TSTON_TOKEN_ID, "get_wallet_address") => Ok(wallet_response.clone().into_bytes()),
                        (TEST_TSTON_WALLET, "get_wallet_data") => Ok(br#"{"exit_code":0,"stack":[{"type":"num","value":"0x3b9aca00"}]}"#.to_vec()),
                        (TONSTAKERS_POOL_ADDRESS, "get_pool_full_data") => Ok(include_bytes!("../../testdata/tonstakers_pool_full_data.json").to_vec()),
                        (_, "get_nft_address_by_index") => Ok(bill_response.clone().into_bytes()),
                        (TEST_BILL, "get_bill_amount") => Ok(br#"{"exit_code":0,"stack":[{"type":"num","value":"0x84e2b0219"}]}"#.to_vec()),
                        call => panic!("unexpected get-method {call:?}"),
                    }
                }),
        );

        let delegations = client.get_staking_delegations(TEST_OWNER.to_string()).await.unwrap();

        assert_eq!(
            delegations.iter().map(|delegation| (delegation.state, delegation.balance.clone(), delegation.shares.clone())).collect::<Vec<_>>(),
            vec![
                (DelegationState::Active, BigUint::from(1_174_894_903u64), BigUint::from(1_000_000_000u64)),
                (DelegationState::Deactivating, BigUint::from(41_909_887_334u64), BigUint::from(35_671_179_801u64)),
            ]
        );
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use primitives::DelegationState;

    use super::*;
    use crate::provider::testkit::create_ton_test_client;

    const TEST_TONSTAKERS_HOLDER: &str = "0:DB7FF3D4D432C072778CCD68E351B2BF55CEE86A13F488EA6E1DBC48C037DA23";

    #[tokio::test]
    async fn test_ton_get_staking_apy_and_validators() -> Result<(), Box<dyn Error + Send + Sync>> {
        let client = create_ton_test_client();
        let apy = client.get_staking_apy().await?.unwrap();
        println!("Tonstakers APY: {apy}");
        assert!(apy > 0.0);

        let validators = client.get_staking_validators(Some(apy)).await?;
        assert_eq!(validators.len(), 1);
        assert_eq!(validators[0].id, TONSTAKERS_POOL_ADDRESS);
        Ok(())
    }

    #[tokio::test]
    async fn test_ton_get_staking_delegations() -> Result<(), Box<dyn Error + Send + Sync>> {
        let client = create_ton_test_client();
        let delegations = client.get_staking_delegations(TEST_TONSTAKERS_HOLDER.to_string()).await?;
        println!("Tonstakers delegations: {delegations:?}");
        let active = delegations.iter().find(|delegation| delegation.state == DelegationState::Active).unwrap();
        assert!(active.balance > active.shares);
        Ok(())
    }
}
