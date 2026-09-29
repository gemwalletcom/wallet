use diesel::prelude::*;
use diesel::upsert::excluded;
use primitives::swap::SwapPartnerTransaction;

use crate::models::NewSwapPartnerTransactionRow;
use crate::{DatabaseClient, DatabaseError};

pub trait SwapPartnerTransactionsRepository {
    fn add_swap_partner_transactions(&mut self, values: Vec<SwapPartnerTransaction>) -> Result<usize, DatabaseError>;
}

impl SwapPartnerTransactionsRepository for DatabaseClient {
    fn add_swap_partner_transactions(&mut self, values: Vec<SwapPartnerTransaction>) -> Result<usize, DatabaseError> {
        use crate::schema::swap_partner_transactions::dsl::*;
        use diesel::query_dsl::methods::FilterDsl;

        if values.is_empty() {
            return Ok(0);
        }
        let rows = values.into_iter().map(NewSwapPartnerTransactionRow::from_primitive).collect::<Vec<_>>();
        let insert = diesel::insert_into(swap_partner_transactions).values(&rows).on_conflict((provider, provider_transaction_id)).do_update().set((
            status.eq(excluded(status)),
            from_address.eq(excluded(from_address)),
            to_address.eq(excluded(to_address)),
            from_asset_id.eq(excluded(from_asset_id)),
            to_asset_id.eq(excluded(to_asset_id)),
            from_value.eq(excluded(from_value)),
            from_amount_usd.eq(excluded(from_amount_usd)),
            to_value.eq(excluded(to_value)),
            to_amount_usd.eq(excluded(to_amount_usd)),
            referral_fee_asset_id.eq(excluded(referral_fee_asset_id)),
            referral_fee_value.eq(excluded(referral_fee_value)),
            referral_fee_amount_usd.eq(excluded(referral_fee_amount_usd)),
            from_transaction_hash.eq(excluded(from_transaction_hash)),
            to_transaction_hash.eq(excluded(to_transaction_hash)),
        ));

        Ok(insert
            .filter(
                status
                    .ne(excluded(status))
                    .or(from_address.ne(excluded(from_address)))
                    .or(to_address.ne(excluded(to_address)))
                    .or(from_asset_id.ne(excluded(from_asset_id)))
                    .or(to_asset_id.ne(excluded(to_asset_id)))
                    .or(from_value.ne(excluded(from_value)))
                    .or(from_amount_usd.is_distinct_from(excluded(from_amount_usd)))
                    .or(to_value.ne(excluded(to_value)))
                    .or(to_amount_usd.is_distinct_from(excluded(to_amount_usd)))
                    .or(referral_fee_asset_id.is_distinct_from(excluded(referral_fee_asset_id)))
                    .or(referral_fee_value.is_distinct_from(excluded(referral_fee_value)))
                    .or(referral_fee_amount_usd.is_distinct_from(excluded(referral_fee_amount_usd)))
                    .or(from_transaction_hash.is_distinct_from(excluded(from_transaction_hash)))
                    .or(to_transaction_hash.is_distinct_from(excluded(to_transaction_hash))),
            )
            .execute(&mut self.connection)?)
    }
}

#[cfg(all(test, feature = "database_integration_tests"))]
mod database_integration_tests {
    use primitives::swap::{SwapPartnerTransaction, SwapStatus};
    use primitives::{Asset, AssetId, Chain, SwapProvider};

    use crate::{AssetsRepository, ChainsRepository, Database, DatabaseError, SwapPartnerTransactionsRepository};

    fn transaction(status: SwapStatus, to_transaction_hash: Option<&str>) -> SwapPartnerTransaction {
        SwapPartnerTransaction {
            provider: SwapProvider::Relay,
            provider_transaction_id: "0x1".to_string(),
            status,
            from_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            to_address: "bc1q4vxn43l44h30nkluqfxd9eckf45vr2awz38lwa".to_string(),
            from_asset_id: AssetId::from_chain(Chain::Ethereum),
            from_value: "1000".to_string(),
            from_amount_usd: Some(1.0),
            to_asset_id: AssetId::from_chain(Chain::Bitcoin),
            to_value: "10".to_string(),
            to_amount_usd: Some(0.99),
            referral_fee: None,
            from_transaction_hash: Some("0xinput".to_string()),
            to_transaction_hash: to_transaction_hash.map(str::to_string),
        }
    }

    #[tokio::test]
    async fn test_add_swap_partner_transactions_writes_only_changes() {
        let database = Database::mock();
        let counts = database
            .run(|client| -> Result<_, DatabaseError> {
                client.add_chains(vec![Chain::Bitcoin, Chain::Ethereum])?;
                client.add_assets(vec![Asset::from_chain(Chain::Bitcoin).as_basic_primitive(), Asset::from_chain(Chain::Ethereum).as_basic_primitive()])?;
                Ok((
                    client.add_swap_partner_transactions(vec![transaction(SwapStatus::Pending, None)])?,
                    client.add_swap_partner_transactions(vec![transaction(SwapStatus::Pending, None)])?,
                    client.add_swap_partner_transactions(vec![transaction(SwapStatus::Completed, Some("output"))])?,
                    client.add_swap_partner_transactions(vec![SwapPartnerTransaction {
                        from_asset_id: AssetId::from_chain(Chain::Bitcoin),
                        ..transaction(SwapStatus::Completed, Some("output"))
                    }])?,
                ))
            })
            .await
            .unwrap();

        assert_eq!(counts, (1, 0, 1, 1));
    }
}
