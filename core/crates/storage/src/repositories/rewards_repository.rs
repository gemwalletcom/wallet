use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use diesel::sql_types::Text;
use primitives::rewards::RewardStatus as PrimitiveRewardStatus;
use primitives::{Chain, NaiveDateTimeExt, ReferralLeader, ReferralLeaderboard, RewardEvent, TransactionState as PrimitiveTransactionState, now};

use crate::models::{NewRewardEventRow, NewRewardReferralRow, NewRewardsRow, NewUsernameRow, ReferralAttemptRow, RewardEventRow, RewardReferralRow, RewardsRow, UsernameRow};
use crate::repositories::transactions_repository::{TransactionFilter, transactions_by_wallet_since};
use crate::repositories::wallets_repository::{device_multicoin_wallet_ids, device_rows_by_wallet_id, first_subscription_date_by_wallet_id, wallet_row_by_id};
use crate::sql_types::{RewardEventType, RewardStatus, UsernameStatus};
use crate::{DatabaseClient, DatabaseError, DieselResultExt};

#[derive(Debug, Clone)]
enum ReferralUpdate {
    VerifiedAt(NaiveDateTime),
}

#[derive(Debug, Clone)]
enum RewardsUpdate {
    Status(RewardStatus),
    VerifyAfter(NaiveDateTime),
    ClearVerifyAfter,
}

#[derive(Debug, Clone)]
pub enum RewardsFilter {
    Username(String),
    Statuses(Vec<PrimitiveRewardStatus>),
    Limit(i64),
}

diesel::define_sql_function!(fn lower(x: Text) -> Text);

pub enum UsernameLookup<'a> {
    Username(&'a str),
    WalletId(i32),
}

fn add_referral(client: &mut DatabaseClient, referral: NewRewardReferralRow) -> Result<(), DieselError> {
    use crate::schema::{rewards, rewards_referrals};
    use diesel::Connection;

    client.connection.transaction(|conn| {
        diesel::insert_into(rewards_referrals::table).values(&referral).execute(conn)?;

        diesel::update(rewards::table.filter(rewards::username.eq(&referral.referred_username)))
            .set(rewards::referrer_username.eq(&referral.referrer_username))
            .execute(conn)?;

        diesel::update(rewards::table.filter(rewards::username.eq(&referral.referrer_username)))
            .set(rewards::referral_count.eq(rewards::referral_count + 1))
            .execute(conn)?;

        Ok(())
    })
}

fn get_referral_by_referred_device_id(client: &mut DatabaseClient, referred_device_id: i32) -> Result<Option<RewardReferralRow>, DieselError> {
    use crate::schema::rewards_referrals::dsl;
    dsl::rewards_referrals
        .filter(dsl::referred_device_id.eq(referred_device_id))
        .select(RewardReferralRow::as_select())
        .first(&mut client.connection)
        .optional()
}

fn get_referral_by_username(client: &mut DatabaseClient, username: &str) -> Result<Option<RewardReferralRow>, DieselError> {
    use crate::schema::rewards_referrals::dsl;
    dsl::rewards_referrals
        .filter(dsl::referred_username.eq(username))
        .select(RewardReferralRow::as_select())
        .first(&mut client.connection)
        .optional()
}

fn update_referral(client: &mut DatabaseClient, referral_id: i32, update: ReferralUpdate) -> Result<(), DieselError> {
    use crate::schema::rewards_referrals::dsl;
    match update {
        ReferralUpdate::VerifiedAt(timestamp) => {
            diesel::update(dsl::rewards_referrals.find(referral_id)).set(dsl::verified_at.eq(timestamp)).execute(&mut client.connection)?;
        }
    }
    Ok(())
}

fn add_referral_attempt(client: &mut DatabaseClient, attempt: ReferralAttemptRow) -> Result<(), DieselError> {
    use crate::schema::rewards_referral_attempts::dsl;
    diesel::insert_into(dsl::rewards_referral_attempts).values(&attempt).execute(&mut client.connection)?;
    Ok(())
}

fn count_referrals_since(client: &mut DatabaseClient, referrer_username: &str, since: NaiveDateTime) -> Result<i64, DieselError> {
    use crate::schema::rewards_referrals::dsl;
    dsl::rewards_referrals
        .filter(dsl::referrer_username.eq(referrer_username))
        .filter(dsl::created_at.ge(since))
        .count()
        .get_result(&mut client.connection)
}

fn get_username(client: &mut DatabaseClient, lookup: UsernameLookup) -> Result<UsernameRow, diesel::result::Error> {
    use crate::schema::usernames::dsl;
    match lookup {
        UsernameLookup::Username(username) => dsl::usernames.filter(lower(dsl::username).eq(username.to_lowercase())).select(UsernameRow::as_select()).first(&mut client.connection),
        UsernameLookup::WalletId(wallet_id) => dsl::usernames.filter(dsl::wallet_id.eq(wallet_id)).select(UsernameRow::as_select()).first(&mut client.connection),
    }
}

fn create_username(client: &mut DatabaseClient, username: NewUsernameRow) -> Result<UsernameRow, diesel::result::Error> {
    use crate::schema::usernames::dsl;
    diesel::insert_into(dsl::usernames).values(&username).returning(UsernameRow::as_returning()).get_result(&mut client.connection)
}

fn update_username(client: &mut DatabaseClient, wallet_id: i32, new_username: &str) -> Result<UsernameRow, diesel::result::Error> {
    use crate::schema::usernames::dsl;
    diesel::update(dsl::usernames.filter(dsl::wallet_id.eq(wallet_id)))
        .set(dsl::username.eq(new_username))
        .returning(UsernameRow::as_returning())
        .get_result(&mut client.connection)
}

pub(crate) fn get_rewards_by_filter(client: &mut DatabaseClient, filters: Vec<RewardsFilter>) -> Result<Vec<RewardsRow>, DieselError> {
    use crate::schema::rewards::dsl;
    let mut query = dsl::rewards.into_boxed();

    for filter in filters {
        match filter {
            RewardsFilter::Username(username) => {
                query = query.filter(dsl::username.eq(username));
            }
            RewardsFilter::Statuses(statuses) => {
                query = query.filter(dsl::status.eq_any(statuses.into_iter().map(RewardStatus::from).collect::<Vec<_>>()));
            }
            RewardsFilter::Limit(limit) => {
                query = query.limit(limit);
            }
        }
    }

    query.select(RewardsRow::as_select()).load(&mut client.connection)
}

fn create_rewards(client: &mut DatabaseClient, rewards: NewRewardsRow) -> Result<RewardsRow, DieselError> {
    use crate::schema::rewards::dsl;
    diesel::insert_into(dsl::rewards).values(&rewards).returning(RewardsRow::as_returning()).get_result(&mut client.connection)
}

fn update_rewards(client: &mut DatabaseClient, username: &str, update: RewardsUpdate) -> Result<usize, DieselError> {
    use crate::schema::rewards::dsl;
    let target = dsl::rewards.filter(dsl::username.eq(username));
    match update {
        RewardsUpdate::Status(status) => diesel::update(target).set(dsl::status.eq(status)).execute(&mut client.connection),
        RewardsUpdate::VerifyAfter(dt) => diesel::update(target).set(dsl::verify_after.eq(dt)).execute(&mut client.connection),
        RewardsUpdate::ClearVerifyAfter => diesel::update(target).set(dsl::verify_after.eq(None::<NaiveDateTime>)).execute(&mut client.connection),
    }
}

fn add_event(client: &mut DatabaseClient, new_event: NewRewardEventRow, points: i32) -> Result<RewardEventRow, DieselError> {
    use crate::schema::{rewards, rewards_events};
    use diesel::Connection;

    if points < 0 {
        return Err(DieselError::RollbackTransaction);
    }

    client.connection.transaction(|conn| {
        let event = diesel::insert_into(rewards_events::table).values(&new_event).returning(RewardEventRow::as_returning()).get_result(conn)?;

        diesel::update(rewards::table.filter(rewards::username.eq(&new_event.username)))
            .set(rewards::points.eq(rewards::points + points))
            .returning(rewards::username)
            .get_result::<String>(conn)?;

        Ok(event)
    })
}

fn get_event(client: &mut DatabaseClient, event_id: i32) -> Result<RewardEventRow, DieselError> {
    use crate::schema::rewards_events::dsl;
    dsl::rewards_events.filter(dsl::id.eq(event_id)).select(RewardEventRow::as_select()).first(&mut client.connection)
}

fn get_events(client: &mut DatabaseClient, username: &str) -> Result<Vec<RewardEventRow>, DieselError> {
    use crate::schema::rewards_events::dsl;
    dsl::rewards_events
        .filter(dsl::username.eq(username))
        .order(dsl::created_at.desc())
        .select(RewardEventRow::as_select())
        .load(&mut client.connection)
}

fn get_top_referrers_since(client: &mut DatabaseClient, event_types: &[RewardEventType], since: NaiveDateTime, limit: i64) -> Result<Vec<(String, i64)>, DieselError> {
    use crate::schema::{rewards, rewards_events};
    use diesel::dsl::count_star;

    rewards_events::table
        .inner_join(rewards::table.on(rewards_events::username.eq(rewards::username)))
        .filter(rewards::status.ne(RewardStatus::Attribution))
        .filter(rewards::status.ne(RewardStatus::Disabled))
        .filter(rewards_events::event_type.eq_any(event_types))
        .filter(rewards_events::created_at.ge(since))
        .group_by(rewards_events::username)
        .select((rewards_events::username, count_star()))
        .order_by(count_star().desc())
        .limit(limit)
        .load(&mut client.connection)
}

fn disable_rewards(client: &mut DatabaseClient, username: &str, reason: &str, comment: &str) -> Result<i32, DieselError> {
    use crate::schema::{rewards, rewards_events};
    use diesel::Connection;

    client.connection.transaction(|conn| {
        diesel::update(rewards::table.filter(rewards::username.eq(username)))
            .set((rewards::status.eq(RewardStatus::Disabled), rewards::disable_reason.eq(reason), rewards::comment.eq(comment)))
            .execute(conn)?;

        let event_id = diesel::insert_into(rewards_events::table)
            .values(NewRewardEventRow {
                username: username.to_string(),
                event_type: RewardEventType::Disabled,
            })
            .returning(rewards_events::id)
            .get_result(conn)?;

        Ok(event_id)
    })
}

fn create_username_and_rewards(client: &mut DatabaseClient, wallet_id: i32, address: &str, device_id: i32) -> Result<RewardsRow, DatabaseError> {
    create_username(
        client,
        NewUsernameRow {
            username: address.to_string(),
            wallet_id,
            status: UsernameStatus::Unverified,
        },
    )?;
    Ok(create_rewards(client, NewRewardsRow::new(address.to_string(), device_id))?)
}

fn find_username(client: &mut DatabaseClient, lookup: UsernameLookup<'_>) -> Result<Option<UsernameRow>, DatabaseError> {
    match get_username(client, lookup) {
        Ok(username) => Ok(Some(username)),
        Err(diesel::result::Error::NotFound) => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn require_username(client: &mut DatabaseClient, lookup: UsernameLookup<'_>) -> Result<UsernameRow, DatabaseError> {
    match lookup {
        UsernameLookup::Username(username) => get_username(client, lookup).or_not_found(username.to_string()),
        UsernameLookup::WalletId(wallet_id) => get_username(client, lookup).or_not_found_internal(wallet_id.to_string()),
    }
}

pub(crate) fn require_rewards(client: &mut DatabaseClient, username: &str) -> Result<RewardsRow, DatabaseError> {
    get_rewards_by_filter(client, vec![RewardsFilter::Username(username.to_string())])?
        .into_iter()
        .next()
        .ok_or_else(|| DatabaseError::not_found("Rewards", username.to_string()))
}

fn require_reward_event(client: &mut DatabaseClient, event_id: i32) -> Result<RewardEventRow, DatabaseError> {
    get_event(client, event_id).or_not_found_internal(event_id.to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RewardsEligibilityConfig {
    pub activity_cutoff: NaiveDateTime,
    pub transactions_required: i64,
}

#[derive(Debug, Clone)]
pub struct ReferrerInfo {
    pub status: PrimitiveRewardStatus,
    pub referral_count: i32,
    pub wallet_id: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReferralRecord {
    pub id: i32,
    pub referrer_username: String,
    pub referred_username: String,
    pub referred_device_id: i32,
    pub verified_at: Option<NaiveDateTime>,
}

impl From<RewardReferralRow> for ReferralRecord {
    fn from(row: RewardReferralRow) -> Self {
        Self {
            id: row.id,
            referrer_username: row.referrer_username,
            referred_username: row.referred_username,
            referred_device_id: row.referred_device_id,
            verified_at: row.verified_at,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RewardsVerification {
    pub status: PrimitiveRewardStatus,
    pub verify_after: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReferralUseFactsRecord {
    pub referred_username: String,
    pub referred_status: Option<PrimitiveRewardStatus>,
    pub wallet_first_subscription_at: Option<NaiveDateTime>,
    pub device_wallets: Vec<(i32, Option<NaiveDateTime>)>,
    pub device_referral: Option<ReferralRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RewardIdentityRecord {
    pub username: String,
    pub wallet_address: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RewardsRecord {
    pub status: PrimitiveRewardStatus,
    pub points: i32,
    pub referral_count: i32,
    pub referrer_username: Option<String>,
    pub disable_reason: Option<String>,
    pub verify_after: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

impl From<RewardsRow> for RewardsRecord {
    fn from(row: RewardsRow) -> Self {
        Self {
            status: *row.status,
            points: row.points,
            referral_count: row.referral_count,
            referrer_username: row.referrer_username,
            disable_reason: row.disable_reason,
            verify_after: row.verify_after,
            created_at: row.created_at,
        }
    }
}

fn latest_wallet_device_id(client: &mut DatabaseClient, wallet_id: i32) -> Result<i32, DatabaseError> {
    device_rows_by_wallet_id(client, wallet_id)?
        .into_iter()
        .max_by_key(|device| device.updated_at)
        .map(|device| device.id)
        .ok_or_else(|| DatabaseError::Error(format!("Wallet {wallet_id} has no subscribed devices")))
}

fn referred_username(client: &mut DatabaseClient, wallet_id: i32) -> Result<String, DatabaseError> {
    match find_username(client, UsernameLookup::WalletId(wallet_id))? {
        Some(username) => Ok(username.username),
        None => {
            let wallet = wallet_row_by_id(client, wallet_id)?;
            Ok(wallet.wallet_id.address().to_string())
        }
    }
}

fn ensure_wallet_reward_identity(client: &mut DatabaseClient, wallet_id: i32) -> Result<RewardIdentityRecord, DatabaseError> {
    let device_id = latest_wallet_device_id(client, wallet_id)?;
    let wallet_address = wallet_row_by_id(client, wallet_id)?.wallet_id.address().to_string();

    let username = match find_username(client, UsernameLookup::WalletId(wallet_id))? {
        Some(username) => {
            if require_rewards(client, &username.username).is_err() {
                create_rewards(client, NewRewardsRow::new(username.username.clone(), device_id))?;
            }
            username.username
        }
        None => {
            create_username_and_rewards(client, wallet_id, &wallet_address, device_id)?;
            require_username(client, UsernameLookup::WalletId(wallet_id))?.username
        }
    };
    Ok(RewardIdentityRecord { username, wallet_address })
}

fn add_referral_verified_event_rows(client: &mut DatabaseClient, referrer_username: &str, referrer_status: &PrimitiveRewardStatus, referred_username: &str) -> Result<Vec<RewardEventRow>, DatabaseError> {
    let mut events = Vec::new();
    if *referrer_status != PrimitiveRewardStatus::Attribution {
        events.push(add_event(
            client,
            NewRewardEventRow {
                username: referrer_username.to_string(),
                event_type: RewardEventType::InviteNew,
            },
            RewardEventType::InviteNew.points(),
        )?);
    }

    let referred_event = add_event(
        client,
        NewRewardEventRow {
            username: referred_username.to_string(),
            event_type: RewardEventType::Joined,
        },
        RewardEventType::Joined.points(),
    )?;
    events.push(referred_event);

    Ok(events)
}

fn add_referral_verified_events(client: &mut DatabaseClient, referrer_username: &str, referrer_status: &PrimitiveRewardStatus, referred_username: &str) -> Result<Vec<RewardEvent>, DatabaseError> {
    Ok(add_referral_verified_event_rows(client, referrer_username, referrer_status, referred_username)?
        .into_iter()
        .map(|event| event.as_primitive())
        .collect())
}

fn add_referral_pending_events(client: &mut DatabaseClient, referrer_username: &str, referrer_status: &PrimitiveRewardStatus) -> Result<Vec<RewardEvent>, DatabaseError> {
    if *referrer_status == PrimitiveRewardStatus::Attribution {
        return Ok(vec![]);
    }
    let event = add_event(
        client,
        NewRewardEventRow {
            username: referrer_username.to_string(),
            event_type: RewardEventType::InvitePending,
        },
        RewardEventType::InvitePending.points(),
    )?;
    Ok(vec![event.as_primitive()])
}

fn add_referral_with_events(
    client: &mut DatabaseClient,
    referrer_username: &str,
    referred_username: &str,
    device_id: i32,
    risk_signal_id: Option<i32>,
    verified_at: Option<NaiveDateTime>,
    referrer_status: &PrimitiveRewardStatus,
) -> Result<Vec<RewardEvent>, DatabaseError> {
    add_referral(
        client,
        NewRewardReferralRow {
            referrer_username: referrer_username.to_string(),
            referred_username: referred_username.to_string(),
            referred_device_id: device_id,
            risk_signal_id,
            verified_at,
        },
    )?;

    if verified_at.is_some() {
        add_referral_verified_events(client, referrer_username, referrer_status, referred_username)
    } else {
        add_referral_pending_events(client, referrer_username, referrer_status)
    }
}

fn complete_referral(client: &mut DatabaseClient, referred_username: &str) -> Result<Vec<i32>, DatabaseError> {
    let Some(referral) = get_referral_by_username(client, referred_username)? else {
        return Ok(vec![]);
    };

    if referral.verified_at.is_some() {
        return Ok(vec![]);
    }

    update_referral(client, referral.id, ReferralUpdate::VerifiedAt(now()))?;
    let referrer_status = *require_rewards(client, &referral.referrer_username)?.status;
    Ok(add_referral_verified_event_rows(client, &referral.referrer_username, &referrer_status, referred_username)?
        .into_iter()
        .map(|event| event.id)
        .collect())
}

pub trait RewardsRepository {
    fn get_rewards_record(&mut self, username: &str) -> Result<RewardsRecord, DatabaseError>;
    fn get_username_by_wallet_id(&mut self, wallet_id: i32) -> Result<Option<String>, DatabaseError>;
    fn get_reward_events_by_wallet_id(&mut self, wallet_id: i32) -> Result<Vec<RewardEvent>, DatabaseError>;
    fn get_reward_event(&mut self, event_id: i32) -> Result<RewardEvent, DatabaseError>;
    fn get_or_add_reward_identity(&mut self, wallet_id: i32) -> Result<RewardIdentityRecord, DatabaseError>;
    fn set_username(&mut self, wallet_id: i32, username: &str) -> Result<i32, DatabaseError>;
    fn get_referral_code(&mut self, code: &str) -> Result<Option<String>, DatabaseError>;
    fn get_referrer_info(&mut self, username: &str) -> Result<ReferrerInfo, DatabaseError>;
    fn get_rewards_verification(&mut self, username: &str) -> Result<RewardsVerification, DatabaseError>;
    fn get_referral_use_facts(&mut self, wallet_id: i32, device_id: i32) -> Result<ReferralUseFactsRecord, DatabaseError>;
    fn get_referral_by_referred_username(&mut self, username: &str) -> Result<Option<ReferralRecord>, DatabaseError>;
    fn delete_rewards_verification_delay(&mut self, username: &str) -> Result<(), DatabaseError>;
    fn set_rewards_verification_delay(&mut self, username: &str, verify_after: NaiveDateTime) -> Result<(), DatabaseError>;
    fn update_referral_verified(&mut self, referral_id: i32, referrer_username: &str, referrer_status: &PrimitiveRewardStatus, referred_username: &str) -> Result<Vec<RewardEvent>, DatabaseError>;
    fn add_referral(
        &mut self,
        referrer_username: &str,
        referred_username: &str,
        device_id: i32,
        risk_signal_id: Option<i32>,
        verified_at: Option<NaiveDateTime>,
        referrer_status: &PrimitiveRewardStatus,
    ) -> Result<Vec<RewardEvent>, DatabaseError>;
    fn add_referral_attempt(&mut self, referrer_username: &str, referred_wallet_id: i32, device_id: i32, risk_signal_id: Option<i32>, reason: &str) -> Result<(), DatabaseError>;
    fn get_wallet_id_by_username(&mut self, username: &str) -> Result<i32, DatabaseError>;
    fn get_referrer_username(&mut self, referred_username: &str) -> Result<Option<String>, DatabaseError>;
    fn get_address_by_username(&mut self, username: &str) -> Result<String, DatabaseError>;
    fn get_referrals_count_since(&mut self, referrer_username: &str, since: NaiveDateTime) -> Result<i64, DatabaseError>;
    fn get_rewards_leaderboard(&mut self) -> Result<ReferralLeaderboard, DatabaseError>;
    fn set_rewards_disabled(&mut self, username: &str, reason: &str, comment: &str) -> Result<i32, DatabaseError>;
    fn get_usernames_by_filter(&mut self, filters: Vec<RewardsFilter>) -> Result<Vec<String>, DatabaseError>;
    fn get_eligible_wallet_id(&mut self, username: &str, eligibility: RewardsEligibilityConfig) -> Result<Option<i32>, DatabaseError>;
    fn update_rewards_verified(&mut self, username: &str) -> Result<Vec<i32>, DatabaseError>;
}

impl RewardsRepository for DatabaseClient {
    fn get_rewards_record(&mut self, username: &str) -> Result<RewardsRecord, DatabaseError> {
        Ok(require_rewards(self, username)?.into())
    }

    fn get_username_by_wallet_id(&mut self, wallet_id: i32) -> Result<Option<String>, DatabaseError> {
        Ok(find_username(self, UsernameLookup::WalletId(wallet_id))?.map(|username| username.username))
    }

    fn get_reward_events_by_wallet_id(&mut self, wallet_id: i32) -> Result<Vec<RewardEvent>, DatabaseError> {
        let username = ensure_wallet_reward_identity(self, wallet_id)?;
        let events = get_events(self, &username.username)?;
        Ok(events.iter().map(RewardEventRow::as_primitive).collect())
    }

    fn get_reward_event(&mut self, event_id: i32) -> Result<RewardEvent, DatabaseError> {
        let event = require_reward_event(self, event_id)?;
        Ok(event.as_primitive())
    }

    fn get_or_add_reward_identity(&mut self, wallet_id: i32) -> Result<RewardIdentityRecord, DatabaseError> {
        ensure_wallet_reward_identity(self, wallet_id)
    }

    fn set_username(&mut self, wallet_id: i32, username: &str) -> Result<i32, DatabaseError> {
        update_username(self, wallet_id, username).or_not_found_internal(wallet_id.to_string())?;

        let event = add_event(
            self,
            NewRewardEventRow {
                username: username.to_string(),
                event_type: RewardEventType::CreateUsername,
            },
            RewardEventType::CreateUsername.points(),
        )?;
        Ok(event.id)
    }

    fn get_referral_code(&mut self, code: &str) -> Result<Option<String>, DatabaseError> {
        Ok(find_username(self, UsernameLookup::Username(code))?.map(|username| username.username))
    }

    fn get_referrer_info(&mut self, username: &str) -> Result<ReferrerInfo, DatabaseError> {
        let username_row = require_username(self, UsernameLookup::Username(username))?;
        let rewards = require_rewards(self, username)?;
        Ok(ReferrerInfo {
            status: *rewards.status,
            referral_count: rewards.referral_count,
            wallet_id: username_row.wallet_id,
        })
    }

    fn get_rewards_verification(&mut self, username: &str) -> Result<RewardsVerification, DatabaseError> {
        let rewards = require_rewards(self, username)?;
        Ok(RewardsVerification {
            status: *rewards.status,
            verify_after: rewards.verify_after,
        })
    }

    fn get_referral_use_facts(&mut self, wallet_id: i32, device_id: i32) -> Result<ReferralUseFactsRecord, DatabaseError> {
        let referred_username = referred_username(self, wallet_id)?;
        let referred_status = self.get_rewards_verification(&referred_username).ok().map(|verification| verification.status);
        let wallet_first_subscription_at = first_subscription_date_by_wallet_id(self, wallet_id)?;
        let device_wallets = device_multicoin_wallet_ids(self, device_id, Chain::Ethereum)?
            .into_iter()
            .map(|wallet_id| Ok((wallet_id, first_subscription_date_by_wallet_id(self, wallet_id)?)))
            .collect::<Result<Vec<_>, DatabaseError>>()?;
        let device_referral = get_referral_by_referred_device_id(self, device_id)?.map(ReferralRecord::from);
        Ok(ReferralUseFactsRecord {
            referred_username,
            referred_status,
            wallet_first_subscription_at,
            device_wallets,
            device_referral,
        })
    }

    fn get_referral_by_referred_username(&mut self, username: &str) -> Result<Option<ReferralRecord>, DatabaseError> {
        Ok(get_referral_by_username(self, username)?.map(ReferralRecord::from))
    }

    fn delete_rewards_verification_delay(&mut self, username: &str) -> Result<(), DatabaseError> {
        update_rewards(self, username, RewardsUpdate::Status(RewardStatus::Unverified))?;
        update_rewards(self, username, RewardsUpdate::ClearVerifyAfter)?;
        Ok(())
    }

    fn set_rewards_verification_delay(&mut self, username: &str, verify_after: NaiveDateTime) -> Result<(), DatabaseError> {
        update_rewards(self, username, RewardsUpdate::VerifyAfter(verify_after))?;
        update_rewards(self, username, RewardsUpdate::Status(RewardStatus::Pending))?;
        Ok(())
    }

    fn update_referral_verified(&mut self, referral_id: i32, referrer_username: &str, referrer_status: &PrimitiveRewardStatus, referred_username: &str) -> Result<Vec<RewardEvent>, DatabaseError> {
        update_referral(self, referral_id, ReferralUpdate::VerifiedAt(now()))?;
        add_referral_verified_events(self, referrer_username, referrer_status, referred_username)
    }

    fn add_referral(
        &mut self,
        referrer_username: &str,
        referred_username: &str,
        device_id: i32,
        risk_signal_id: Option<i32>,
        verified_at: Option<NaiveDateTime>,
        referrer_status: &PrimitiveRewardStatus,
    ) -> Result<Vec<RewardEvent>, DatabaseError> {
        add_referral_with_events(self, referrer_username, referred_username, device_id, risk_signal_id, verified_at, referrer_status)
    }

    fn add_referral_attempt(&mut self, referrer_username: &str, wallet_id: i32, device_id: i32, risk_signal_id: Option<i32>, reason: &str) -> Result<(), DatabaseError> {
        add_referral_attempt(
            self,
            ReferralAttemptRow {
                referrer_username: referrer_username.to_string(),
                wallet_id,
                device_id,
                risk_signal_id,
                reason: reason.to_string(),
            },
        )?;
        Ok(())
    }

    fn get_wallet_id_by_username(&mut self, username: &str) -> Result<i32, DatabaseError> {
        let username = require_username(self, UsernameLookup::Username(username))?;
        Ok(username.wallet_id)
    }

    fn get_referrer_username(&mut self, referred_username: &str) -> Result<Option<String>, DatabaseError> {
        let referral = get_referral_by_username(self, referred_username)?;
        Ok(referral.map(|r| r.referrer_username))
    }

    fn get_address_by_username(&mut self, username: &str) -> Result<String, DatabaseError> {
        let username_row = require_username(self, UsernameLookup::Username(username))?;
        let wallet = wallet_row_by_id(self, username_row.wallet_id)?;
        Ok(wallet.wallet_id.address().to_string())
    }

    fn get_referrals_count_since(&mut self, referrer_username: &str, since: NaiveDateTime) -> Result<i64, DatabaseError> {
        Ok(count_referrals_since(self, referrer_username, since)?)
    }

    fn get_rewards_leaderboard(&mut self) -> Result<ReferralLeaderboard, DatabaseError> {
        let current = now();
        let limit = 10;
        let invite_types = [RewardEventType::InviteNew];
        let points_per_referral = RewardEventType::InviteNew.points() as i64;

        let map_entry = |(username, referrals): (String, i64)| ReferralLeader {
            username,
            referrals: referrals as i32,
            points: (referrals * points_per_referral) as i32,
        };

        let daily = get_top_referrers_since(self, &invite_types, current.days_ago(1), limit)?.into_iter().map(map_entry).collect();

        let weekly = get_top_referrers_since(self, &invite_types, current.days_ago(7), limit)?.into_iter().map(map_entry).collect();

        let monthly = get_top_referrers_since(self, &invite_types, current.days_ago(30), limit)?.into_iter().map(map_entry).collect();

        Ok(ReferralLeaderboard { daily, weekly, monthly })
    }

    fn set_rewards_disabled(&mut self, username: &str, reason: &str, comment: &str) -> Result<i32, DatabaseError> {
        Ok(disable_rewards(self, username, reason, comment)?)
    }

    fn get_usernames_by_filter(&mut self, filters: Vec<RewardsFilter>) -> Result<Vec<String>, DatabaseError> {
        Ok(get_rewards_by_filter(self, filters)?.into_iter().map(|row| row.username).collect())
    }

    fn get_eligible_wallet_id(&mut self, username: &str, eligibility: RewardsEligibilityConfig) -> Result<Option<i32>, DatabaseError> {
        let username_row = require_username(self, UsernameLookup::Username(username))?;
        let rewards = require_rewards(self, &username_row.username)?;

        if *rewards.status != PrimitiveRewardStatus::Unverified {
            return Ok(None);
        }

        if rewards.verify_after.is_some_and(|dt| dt > now()) {
            return Ok(None);
        }

        let Some(first_subscription_at) = first_subscription_date_by_wallet_id(self, username_row.wallet_id)? else {
            return Ok(None);
        };

        if first_subscription_at > eligibility.activity_cutoff {
            return Ok(None);
        }

        let Some(latest_activity_at) = device_rows_by_wallet_id(self, username_row.wallet_id)?.into_iter().map(|device| device.updated_at).max() else {
            return Ok(None);
        };

        if latest_activity_at < eligibility.activity_cutoff {
            return Ok(None);
        }

        let transactions_current = transactions_by_wallet_since(self, username_row.wallet_id, first_subscription_at, vec![TransactionFilter::States(vec![PrimitiveTransactionState::Confirmed])])?.len() as i64;

        if transactions_current < eligibility.transactions_required {
            return Ok(None);
        }

        Ok(Some(username_row.wallet_id))
    }

    fn update_rewards_verified(&mut self, username: &str) -> Result<Vec<i32>, DatabaseError> {
        update_rewards(self, username, RewardsUpdate::Status(RewardStatus::Verified))?;

        let enabled_event = add_event(
            self,
            NewRewardEventRow {
                username: username.to_string(),
                event_type: RewardEventType::Enabled,
            },
            RewardEventType::Enabled.points(),
        )?;

        let mut event_ids = vec![enabled_event.id];
        event_ids.extend(complete_referral(self, username)?);
        Ok(event_ids)
    }
}
