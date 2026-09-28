use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use primitives::rewards::{RedemptionStatus as PrimitiveRedemptionStatus, RewardRedemption, RewardRedemptionOption, RewardRedemptionType as PrimitiveRewardRedemptionType};

use crate::models::{AssetRow, NewRewardRedemptionRow, RedemptionOptionFull, RewardRedemptionOptionRow, RewardRedemptionRow};
use crate::repositories::rewards_repository::{RewardsFilter, get_rewards_by_filter};
use crate::sql_types::{RedemptionStatus, RewardRedemptionType};
use crate::{DatabaseClient, DatabaseError, DieselResultExt};

#[derive(Debug, Clone, PartialEq)]
pub struct RedemptionRecord {
    pub username: String,
    pub option_id: String,
    pub wallet_id: i32,
    pub status: PrimitiveRedemptionStatus,
}

#[derive(Debug, Clone)]
pub enum RedemptionUpdate {
    Status(PrimitiveRedemptionStatus),
    TransactionId(String),
    Error(String),
}

pub trait RewardsRedemptionsRepository {
    fn add_redemption(&mut self, username: &str, option_id: &str, device_id: i32, wallet_id: i32) -> Result<RewardRedemption, DatabaseError>;
    fn get_redemption(&mut self, redemption_id: i32) -> Result<RedemptionRecord, DatabaseError>;
    fn update_redemption(&mut self, redemption_id: i32, updates: Vec<RedemptionUpdate>) -> Result<(), DatabaseError>;
    fn get_redemption_options(&mut self, types: &[PrimitiveRewardRedemptionType]) -> Result<Vec<RewardRedemptionOption>, DatabaseError>;
    fn get_redemption_option(&mut self, id: &str) -> Result<RewardRedemptionOption, DatabaseError>;
    fn count_redemptions_since(&mut self, username: &str, since: NaiveDateTime) -> Result<i64, DatabaseError>;
}

fn insert_redemption(client: &mut DatabaseClient, username: &str, points: i32, redemption: NewRewardRedemptionRow) -> Result<i32, DieselError> {
    use crate::schema::{rewards, rewards_redemption_options, rewards_redemptions};
    use diesel::Connection;

    if points < 0 {
        return Err(DieselError::RollbackTransaction);
    }

    client.connection.transaction(|conn| {
        let rows_updated = diesel::update(
            rewards_redemption_options::table.filter(
                rewards_redemption_options::id
                    .eq(&redemption.option_id)
                    .and(rewards_redemption_options::remaining.is_null().or(rewards_redemption_options::remaining.gt(0))),
            ),
        )
        .set(rewards_redemption_options::remaining.eq(rewards_redemption_options::remaining - 1))
        .execute(conn)?;

        if rows_updated == 0 {
            return Err(DieselError::NotFound);
        }

        if points > 0 {
            diesel::update(rewards::table.filter(rewards::username.eq(username).and(rewards::points.ge(points))))
                .set(rewards::points.eq(rewards::points - points))
                .returning(rewards::username)
                .get_result::<String>(conn)?;
        }

        diesel::insert_into(rewards_redemptions::table).values(&redemption).returning(rewards_redemptions::id).get_result(conn)
    })
}

fn redemption_row(client: &mut DatabaseClient, redemption_id: i32) -> Result<RewardRedemptionRow, DieselError> {
    use crate::schema::rewards_redemptions::dsl;
    dsl::rewards_redemptions.filter(dsl::id.eq(redemption_id)).select(RewardRedemptionRow::as_select()).first(&mut client.connection)
}

fn redemption_option(client: &mut DatabaseClient, id: &str) -> Result<RedemptionOptionFull, DieselError> {
    use crate::schema::{assets, rewards_redemption_options};
    rewards_redemption_options::table
        .filter(rewards_redemption_options::id.eq(id))
        .left_join(assets::table.on(rewards_redemption_options::asset_id.eq(assets::id.nullable())))
        .select((RewardRedemptionOptionRow::as_select(), Option::<AssetRow>::as_select()))
        .first::<(RewardRedemptionOptionRow, Option<AssetRow>)>(&mut client.connection)
        .map(|(option, asset)| RedemptionOptionFull::new(option, asset))
}

impl RewardsRedemptionsRepository for DatabaseClient {
    fn add_redemption(&mut self, username: &str, option_id: &str, device_id: i32, wallet_id: i32) -> Result<RewardRedemption, DatabaseError> {
        let redemption_option = redemption_option(self, option_id).or_not_found(option_id.to_string())?;
        let rewards = get_rewards_by_filter(self, vec![RewardsFilter::Username(username.to_string())])?
            .into_iter()
            .next()
            .ok_or_else(|| DatabaseError::not_found("Rewards", username.to_string()))?;

        if rewards.points < redemption_option.option.points {
            return Err(DatabaseError::Error("Not enough points".into()));
        }

        if redemption_option.option.remaining == Some(0) {
            return Err(DatabaseError::Error("Redemption option is no longer available".into()));
        }

        let redemption_id = insert_redemption(
            self,
            username,
            redemption_option.option.points,
            NewRewardRedemptionRow {
                username: username.to_string(),
                option_id: option_id.to_string(),
                device_id,
                wallet_id,
                status: RedemptionStatus::Pending,
            },
        )?;

        let option = redemption_option.as_primitive()?;
        let redemption_row = redemption_row(self, redemption_id).or_not_found_internal(redemption_id.to_string())?;
        Ok(redemption_row.as_primitive(option))
    }

    fn get_redemption(&mut self, redemption_id: i32) -> Result<RedemptionRecord, DatabaseError> {
        let row = redemption_row(self, redemption_id).or_not_found_internal(redemption_id.to_string())?;
        Ok(RedemptionRecord {
            username: row.username,
            option_id: row.option_id,
            wallet_id: row.wallet_id,
            status: row.status.0,
        })
    }

    fn update_redemption(&mut self, redemption_id: i32, updates: Vec<RedemptionUpdate>) -> Result<(), DatabaseError> {
        use crate::schema::rewards_redemptions::dsl;

        if updates.is_empty() {
            return Ok(());
        }

        for update in updates {
            let target = dsl::rewards_redemptions.find(redemption_id);
            match update {
                RedemptionUpdate::Status(value) => diesel::update(target).set(dsl::status.eq(RedemptionStatus::from(value))).execute(&mut self.connection)?,
                RedemptionUpdate::TransactionId(value) => diesel::update(target).set(dsl::transaction_id.eq(value)).execute(&mut self.connection)?,
                RedemptionUpdate::Error(value) => diesel::update(target).set(dsl::error.eq(value)).execute(&mut self.connection)?,
            };
        }

        Ok(())
    }

    fn get_redemption_options(&mut self, types: &[PrimitiveRewardRedemptionType]) -> Result<Vec<RewardRedemptionOption>, DatabaseError> {
        use crate::schema::{assets, rewards_redemption_options};
        let types: Vec<RewardRedemptionType> = types.iter().copied().map(RewardRedemptionType::from).collect();
        let results: Vec<(RewardRedemptionOptionRow, Option<AssetRow>)> = rewards_redemption_options::table
            .filter(rewards_redemption_options::redemption_type.eq_any(types))
            .left_join(assets::table.on(rewards_redemption_options::asset_id.eq(assets::id.nullable())))
            .select((RewardRedemptionOptionRow::as_select(), Option::<AssetRow>::as_select()))
            .load(&mut self.connection)?;
        results.into_iter().map(|(option, asset)| RedemptionOptionFull::new(option, asset).as_primitive()).collect()
    }

    fn get_redemption_option(&mut self, id: &str) -> Result<RewardRedemptionOption, DatabaseError> {
        redemption_option(self, id).or_not_found(id.to_string())?.as_primitive()
    }

    fn count_redemptions_since(&mut self, username: &str, since: NaiveDateTime) -> Result<i64, DatabaseError> {
        use crate::schema::rewards_redemptions::dsl;
        Ok(dsl::rewards_redemptions.filter(dsl::username.eq(username)).filter(dsl::created_at.ge(since)).count().get_result(&mut self.connection)?)
    }
}
