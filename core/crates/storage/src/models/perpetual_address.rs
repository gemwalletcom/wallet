use chrono::NaiveDateTime;
use diesel::prelude::*;
use primitives::perpetual::PerpetualAddress;

use crate::sql_types::PerpetualProviderRow;

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::perpetuals_addresses)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct NewPerpetualAddressRow {
    pub provider: PerpetualProviderRow,
    pub address_id: i32,
    pub referred_at: NaiveDateTime,
    pub cumulative_volume_usd: f64,
    pub cumulative_referral_earned_usd: f64,
}

impl NewPerpetualAddressRow {
    pub(crate) fn from_primitive(address_id: i32, address: PerpetualAddress) -> Self {
        Self {
            provider: address.provider.into(),
            address_id,
            referred_at: address.referred_at.naive_utc(),
            cumulative_volume_usd: address.cumulative_volume_usd,
            cumulative_referral_earned_usd: address.cumulative_referral_earned_usd,
        }
    }
}
