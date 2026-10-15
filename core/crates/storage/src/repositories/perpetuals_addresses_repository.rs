use chrono::Utc;
use diesel::{prelude::*, upsert::excluded};
use primitives::perpetual::PerpetualAddress;

use crate::models::NewPerpetualAddressRow;
use crate::schema::perpetuals_addresses;
use crate::{DatabaseClient, DatabaseError, WalletsRepository};

pub trait PerpetualsAddressesRepository {
    fn set_perpetual_addresses(&mut self, addresses: Vec<PerpetualAddress>) -> Result<usize, DatabaseError>;
}

impl PerpetualsAddressesRepository for DatabaseClient {
    fn set_perpetual_addresses(&mut self, addresses: Vec<PerpetualAddress>) -> Result<usize, DatabaseError> {
        if addresses.is_empty() {
            return Ok(0);
        }
        let address_ids = self.add_wallet_addresses(addresses.iter().map(|address| address.address.clone()).collect())?;
        let rows = addresses
            .into_iter()
            .map(|address| {
                let address_id = address_ids[&address.address];
                NewPerpetualAddressRow::from_primitive(address_id, address)
            })
            .collect::<Vec<_>>();
        Ok(diesel::insert_into(perpetuals_addresses::table)
            .values(&rows)
            .on_conflict((perpetuals_addresses::provider, perpetuals_addresses::address_id))
            .do_update()
            .set((
                perpetuals_addresses::referred_at.eq(excluded(perpetuals_addresses::referred_at)),
                perpetuals_addresses::cumulative_volume_usd.eq(excluded(perpetuals_addresses::cumulative_volume_usd)),
                perpetuals_addresses::cumulative_referral_earned_usd.eq(excluded(perpetuals_addresses::cumulative_referral_earned_usd)),
                perpetuals_addresses::updated_at.eq(Utc::now().naive_utc()),
            ))
            .execute(&mut self.connection)?)
    }
}

#[cfg(all(test, feature = "database_integration_tests"))]
mod database_integration_tests {
    use chrono::{DateTime, NaiveDateTime};
    use primitives::PerpetualProvider;

    use super::*;
    use crate::Database;
    use crate::schema::wallets_addresses;

    #[tokio::test]
    async fn test_set_perpetual_addresses_keeps_unsubscribed_accounts_and_creation_time() {
        let database = Database::mock();
        let address = PerpetualAddress {
            provider: PerpetualProvider::Hypercore,
            address: "0x0000000000000000000000000000000000000001".to_string(),
            referred_at: DateTime::from_timestamp(1, 0).unwrap(),
            cumulative_volume_usd: 100.0,
            cumulative_referral_earned_usd: 1.0,
        };
        let initial = address.clone();
        database
            .transaction(move |client| {
                assert_eq!(client.set_perpetual_addresses(vec![initial])?, 1);
                diesel::update(perpetuals_addresses::table)
                    .set((perpetuals_addresses::created_at.eq(DateTime::UNIX_EPOCH.naive_utc()), perpetuals_addresses::updated_at.eq(DateTime::UNIX_EPOCH.naive_utc())))
                    .execute(&mut client.connection)?;
                Ok::<_, DatabaseError>(())
            })
            .await
            .unwrap();
        database
            .transaction(move |client| {
                assert_eq!(
                    client.set_perpetual_addresses(vec![PerpetualAddress {
                        cumulative_volume_usd: 200.0,
                        cumulative_referral_earned_usd: 2.0,
                        ..address
                    }])?,
                    1
                );
                assert_eq!(client.set_perpetual_addresses(vec![])?, 0);
                let rows = perpetuals_addresses::table
                    .inner_join(wallets_addresses::table)
                    .select((
                        wallets_addresses::address,
                        perpetuals_addresses::cumulative_volume_usd,
                        perpetuals_addresses::cumulative_referral_earned_usd,
                        perpetuals_addresses::created_at,
                    ))
                    .load::<(String, f64, f64, NaiveDateTime)>(&mut client.connection)?;
                assert_eq!(rows, vec![("0x0000000000000000000000000000000000000001".to_string(), 200.0, 2.0, DateTime::UNIX_EPOCH.naive_utc())]);
                let refreshed = perpetuals_addresses::table
                    .select(perpetuals_addresses::updated_at.gt(perpetuals_addresses::created_at))
                    .get_result::<bool>(&mut client.connection)?;
                assert!(refreshed);
                Ok::<_, DatabaseError>(())
            })
            .await
            .unwrap();
    }
}
