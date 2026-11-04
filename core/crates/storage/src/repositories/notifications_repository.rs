use chrono::NaiveDateTime;
use diesel::prelude::*;
use primitives::{AssetId, NotificationData, NotificationType};

use crate::models::{AssetRow, NewNotificationRow, NotificationRow};
use crate::schema::{assets, devices, notifications, wallets, wallets_subscriptions};
use crate::{DatabaseClient, DatabaseError};

type WalletIdsSubquery<'a> = diesel::dsl::Select<diesel::dsl::Filter<diesel::dsl::InnerJoin<wallets_subscriptions::table, devices::table>, diesel::dsl::Eq<devices::identifier, &'a str>>, wallets_subscriptions::wallet_id>;

fn wallet_ids_by_device_id(device_id: &str) -> WalletIdsSubquery<'_> {
    wallets_subscriptions::table.inner_join(devices::table).filter(devices::identifier.eq(device_id)).select(wallets_subscriptions::wallet_id)
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewNotification {
    pub wallet_id: i32,
    pub asset_id: Option<AssetId>,
    pub notification_type: NotificationType,
    pub metadata: Option<serde_json::Value>,
}

pub trait NotificationsRepository {
    fn get_notifications_by_device_id(&mut self, device_id: &str, from_datetime: Option<NaiveDateTime>, limit: usize) -> Result<Vec<NotificationData>, DatabaseError>;
    fn add_notifications(&mut self, notifications: Vec<NewNotification>) -> Result<usize, DatabaseError>;
    fn set_notifications_read(&mut self, wallet_id: i32) -> Result<usize, DatabaseError>;
}

impl NotificationsRepository for DatabaseClient {
    fn get_notifications_by_device_id(&mut self, device_id: &str, from_datetime: Option<NaiveDateTime>, limit: usize) -> Result<Vec<NotificationData>, DatabaseError> {
        let mut query = notifications::table
            .inner_join(wallets::table)
            .left_join(assets::table)
            .filter(notifications::wallet_id.eq_any(wallet_ids_by_device_id(device_id)))
            .order(notifications::created_at.desc())
            .select((NotificationRow::as_select(), wallets::identifier, Option::<AssetRow>::as_select()))
            .into_boxed();

        if let Some(datetime) = from_datetime {
            query = query.filter(notifications::created_at.gt(datetime));
        }

        let rows: Vec<(NotificationRow, String, Option<AssetRow>)> = query.limit(limit as i64).load(&mut self.connection)?;
        Ok(rows.into_iter().map(|(row, wallet_identifier, asset_row)| row.as_primitive(wallet_identifier, asset_row.map(|a| a.as_primitive()))).collect())
    }

    fn add_notifications(&mut self, values: Vec<NewNotification>) -> Result<usize, DatabaseError> {
        let rows: Vec<NewNotificationRow> = values
            .into_iter()
            .map(|value| NewNotificationRow {
                wallet_id: value.wallet_id,
                asset_id: value.asset_id.map(Into::into),
                notification_type: value.notification_type.into(),
                metadata: value.metadata,
            })
            .collect();
        Ok(diesel::insert_into(notifications::table).values(&rows).execute(&mut self.connection)?)
    }

    fn set_notifications_read(&mut self, wallet_id: i32) -> Result<usize, DatabaseError> {
        Ok(diesel::update(notifications::table)
            .filter(notifications::wallet_id.eq(wallet_id))
            .filter(notifications::is_read.eq(false))
            .set((notifications::is_read.eq(true), notifications::read_at.eq(diesel::dsl::now)))
            .execute(&mut self.connection)?)
    }
}

#[cfg(all(test, feature = "database_integration_tests"))]
mod database_integration_tests {
    use diesel::prelude::*;

    use super::{NewNotification, NotificationsRepository};
    use crate::schema::notifications;
    use crate::{Database, DatabaseError, NewWallet, WalletsRepository};

    #[tokio::test]
    async fn test_marking_a_wallet_read_leaves_other_wallets_unread() {
        let (opened, other, read) = Database::mock()
            .run(|client| -> Result<_, DatabaseError> {
                let opened = client.get_or_create_wallet(NewWallet::mock("0xopened"))?.id;
                let other = client.get_or_create_wallet(NewWallet::mock("0xother"))?.id;
                client.add_notifications(vec![NewNotification::mock(opened), NewNotification::mock(other)])?;

                client.set_notifications_read(opened)?;

                let read = notifications::table
                    .filter(notifications::wallet_id.eq_any([opened, other]))
                    .select((notifications::wallet_id, notifications::is_read))
                    .order(notifications::wallet_id)
                    .load::<(i32, bool)>(&mut client.connection)?;
                Ok((opened, other, read))
            })
            .await
            .unwrap();

        assert_eq!(read, vec![(opened, true), (other, false)]);
    }
}
