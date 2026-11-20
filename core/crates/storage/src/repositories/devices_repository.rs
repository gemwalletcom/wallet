use diesel::{prelude::*, upsert::excluded};
use primitives::{DateTime, Device, DurationExt};
use std::time::Duration;

use crate::models::{DeviceRow, UpdateDeviceRow};
use crate::repositories::wallets_repository::delete_subscriptions_for_device_ids;
use crate::repositories::{Condition, QueryFilter, matching};
use crate::sql_types::DateTimeRow;
use crate::{DatabaseClient, DatabaseError, DieselResultExt};

#[derive(Debug, Clone, PartialEq)]
pub enum DeviceUpdate {
    IsPushEnabled(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeviceFilter {
    Ids(Vec<String>),
    IsPushEnabled(bool),
    CreatedBetween { start: DateTime, end: DateTime },
}

#[derive(Debug, Clone)]
pub struct DeviceRecord {
    pub id: i32,
    pub device: Device,
    pub created_at: DateTime,
}

impl DeviceRecord {
    pub(crate) fn from_row(row: DeviceRow) -> Self {
        Self {
            id: row.id,
            device: row.as_primitive(),
            created_at: row.created_at.into(),
        }
    }
}

impl QueryFilter<crate::schema::devices::table> for DeviceFilter {
    fn condition(self) -> Condition<crate::schema::devices::table> {
        use crate::schema::devices::dsl::*;
        match self {
            DeviceFilter::Ids(values) => Box::new(identifier.eq_any(values)),
            DeviceFilter::IsPushEnabled(enabled) => Box::new(is_push_enabled.eq(enabled)),
            DeviceFilter::CreatedBetween { start, end } => Box::new(
                created_at
                    .between(DateTimeRow::from(start), DateTimeRow::from(end))
                    .and(diesel::dsl::sql::<diesel::sql_types::Bool>("DATE_TRUNC('hour', updated_at) = DATE_TRUNC('hour', created_at)")),
            ),
        }
    }
}

pub trait DevicesRepository {
    fn add_device(&mut self, device: Device) -> Result<Device, DatabaseError>;
    fn get_device_record(&mut self, device_id: &str) -> Result<DeviceRecord, DatabaseError>;
    fn update_device(&mut self, device: Device) -> Result<Device, DatabaseError>;
    fn update_devices(&mut self, filters: Vec<DeviceFilter>, updates: Vec<DeviceUpdate>) -> Result<usize, DatabaseError>;
    fn delete_devices_subscriptions_after_days(&mut self, days: u64) -> Result<usize, DatabaseError>;
    fn get_inactive_devices(&mut self, min_days: u64, max_days: u64, push_enabled: Option<bool>) -> Result<Vec<Device>, DatabaseError>;
}

pub(crate) fn device_row(client: &mut DatabaseClient, device_id_value: &str) -> Result<DeviceRow, diesel::result::Error> {
    use crate::schema::devices::dsl::*;
    devices.filter(identifier.eq(device_id_value)).select(DeviceRow::as_select()).first(&mut client.connection)
}

impl DevicesRepository for DatabaseClient {
    fn add_device(&mut self, device: Device) -> Result<Device, DatabaseError> {
        use crate::schema::devices::dsl::*;
        let device = UpdateDeviceRow::from_primitive(device);
        Ok(diesel::insert_into(devices)
            .values(&device)
            .on_conflict(identifier)
            .do_update()
            .set((identifier.eq(excluded(identifier)),))
            .returning(DeviceRow::as_returning())
            .get_result(&mut self.connection)?
            .as_primitive())
    }

    fn get_device_record(&mut self, device_id: &str) -> Result<DeviceRecord, DatabaseError> {
        Ok(DeviceRecord::from_row(device_row(self, device_id).or_not_found(device_id.to_string())?))
    }

    fn update_device(&mut self, device: Device) -> Result<Device, DatabaseError> {
        let device = UpdateDeviceRow::from_primitive(device);
        let device_id_value = device.identifier.clone();
        use crate::schema::devices::dsl::*;
        Ok(diesel::update(devices)
            .filter(identifier.eq(device_id_value.clone()))
            .set(device)
            .returning(DeviceRow::as_returning())
            .get_result(&mut self.connection)
            .or_not_found(device_id_value)?
            .as_primitive())
    }

    fn update_devices(&mut self, filters: Vec<DeviceFilter>, updates: Vec<DeviceUpdate>) -> Result<usize, DatabaseError> {
        use crate::schema::devices::dsl::*;
        Ok(updates.into_iter().try_fold(0, |total, update| {
            let target = devices.filter(matching(filters.clone()));
            let updated = match update {
                DeviceUpdate::IsPushEnabled(value) => diesel::update(target).set(is_push_enabled.eq(value)).execute(&mut self.connection)?,
            };
            Ok::<_, diesel::result::Error>(total + updated)
        })?)
    }

    fn delete_devices_subscriptions_after_days(&mut self, days: u64) -> Result<usize, DatabaseError> {
        let cutoff_date = DateTime::now() - Duration::days(days);
        self.transaction(|client| {
            use crate::schema::devices::dsl::*;
            let device_ids: Vec<i32> = devices.filter(updated_at.lt(DateTimeRow::from(cutoff_date))).select(id).load(&mut client.connection)?;
            diesel::update(devices.filter(id.eq_any(&device_ids)).filter(subscriptions_version.ne(0)))
                .set(subscriptions_version.eq(0))
                .execute(&mut client.connection)?;
            Ok(delete_subscriptions_for_device_ids(client, device_ids)?)
        })
    }

    fn get_inactive_devices(&mut self, min_days: u64, max_days: u64, push_enabled: Option<bool>) -> Result<Vec<Device>, DatabaseError> {
        let min_days_cutoff = DateTime::now() - Duration::days(min_days);
        let max_days_cutoff = DateTime::now() - Duration::days(max_days);

        let filters = [DeviceFilter::CreatedBetween {
            start: max_days_cutoff,
            end: min_days_cutoff,
        }]
        .into_iter()
        .chain(push_enabled.map(DeviceFilter::IsPushEnabled))
        .collect();
        Ok(crate::schema::devices::table
            .filter(matching(filters))
            .select(DeviceRow::as_select())
            .load(&mut self.connection)?
            .into_iter()
            .map(|x| x.as_primitive())
            .collect())
    }
}

#[cfg(all(test, feature = "database_integration_tests"))]
mod database_integration_tests {
    use diesel::prelude::*;
    use primitives::{Currency, DateTime, Device, DurationExt, FiatRate, FiatRateProvider};
    use std::time::Duration;

    use crate::sql_types::DateTimeRow;
    use crate::{Database, DatabaseError, DevicesRepository, FiatRepository};

    #[tokio::test]
    async fn test_get_device_record_missing_row_is_not_found() {
        let error = Database::mock().run(|client| client.get_device_record("missing-device")).await.unwrap_err();

        assert!(error.is_not_found());
    }

    #[tokio::test]
    async fn test_dropping_dormant_subscriptions_resets_the_subscriptions_version() {
        let versions = Database::mock()
            .run(|client| {
                client.set_fiat_rates(FiatRateProvider::Coingecko, vec![FiatRate { symbol: Currency::USD, rate: 1.0 }])?;
                client.add_device(Device { id: "dormant".to_string(), ..Device::mock() })?;
                client.add_device(Device { id: "active".to_string(), ..Device::mock() })?;
                {
                    use crate::schema::devices::dsl::*;
                    diesel::update(devices.filter(identifier.eq("dormant")))
                        .set(updated_at.eq(DateTimeRow::from(DateTime::now() - Duration::days(121))))
                        .execute(&mut client.connection)?;
                }

                client.delete_devices_subscriptions_after_days(120)?;

                Ok::<_, DatabaseError>((client.get_device_record("dormant")?.device.subscriptions_version, client.get_device_record("active")?.device.subscriptions_version))
            })
            .await
            .unwrap();

        assert_eq!(versions, (0, 1));
    }
}
