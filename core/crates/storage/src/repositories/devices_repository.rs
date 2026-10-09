use chrono::{Duration, NaiveDateTime, Utc};
use diesel::{prelude::*, upsert::excluded};
use primitives::Device;

use crate::models::{DeviceRow, UpdateDeviceRow};
use crate::repositories::wallets_repository::delete_subscriptions_for_device_ids;
use crate::repositories::{Condition, QueryFilter, matching};
use crate::{DatabaseClient, DatabaseError, DieselResultExt};

#[derive(Debug, Clone, PartialEq)]
pub enum DeviceUpdate {
    IsPushEnabled(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeviceFilter {
    Ids(Vec<String>),
    IsPushEnabled(bool),
    CreatedBetween { start: NaiveDateTime, end: NaiveDateTime },
}

#[derive(Debug, Clone)]
pub struct DeviceRecord {
    pub id: i32,
    pub device: Device,
    pub created_at: NaiveDateTime,
}

impl DeviceRecord {
    pub(crate) fn from_row(row: DeviceRow) -> Self {
        Self {
            id: row.id,
            device: row.as_primitive(),
            created_at: row.created_at,
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
                    .between(start, end)
                    .and(diesel::dsl::sql::<diesel::sql_types::Bool>("DATE_TRUNC('hour', updated_at) = DATE_TRUNC('hour', created_at)")),
            ),
        }
    }
}

pub trait DevicesRepository {
    fn add_device(&mut self, device: Device) -> Result<Device, DatabaseError>;
    fn get_device(&mut self, device_id: &str) -> Result<Device, DatabaseError>;
    fn get_device_record(&mut self, device_id: &str) -> Result<DeviceRecord, DatabaseError>;
    fn get_device_exist(&mut self, device_id: &str) -> Result<bool, DatabaseError>;
    fn get_device_row_id(&mut self, device_id: &str) -> Result<i32, DatabaseError>;
    fn update_device(&mut self, device: Device) -> Result<Device, DatabaseError>;
    fn update_devices(&mut self, filters: Vec<DeviceFilter>, updates: Vec<DeviceUpdate>) -> Result<usize, DatabaseError>;
    fn delete_devices_subscriptions_after_days(&mut self, days: i64) -> Result<usize, DatabaseError>;
    fn get_inactive_devices(&mut self, min_days: i64, max_days: i64, push_enabled: Option<bool>) -> Result<Vec<Device>, DatabaseError>;
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

    fn get_device(&mut self, device_id: &str) -> Result<Device, DatabaseError> {
        Ok(device_row(self, device_id).or_not_found(device_id.to_string())?.as_primitive())
    }

    fn get_device_record(&mut self, device_id: &str) -> Result<DeviceRecord, DatabaseError> {
        Ok(DeviceRecord::from_row(device_row(self, device_id).or_not_found(device_id.to_string())?))
    }

    fn get_device_exist(&mut self, device_id: &str) -> Result<bool, DatabaseError> {
        match device_row(self, device_id) {
            Ok(_) => Ok(true),
            Err(diesel::result::Error::NotFound) => Ok(false),
            Err(error) => Err(error.into()),
        }
    }

    fn get_device_row_id(&mut self, device_id: &str) -> Result<i32, DatabaseError> {
        Ok(device_row(self, device_id).or_not_found(device_id.to_string())?.id)
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

    fn delete_devices_subscriptions_after_days(&mut self, days: i64) -> Result<usize, DatabaseError> {
        let cutoff_date = Utc::now() - Duration::days(days);
        let device_ids: Vec<i32> = {
            use crate::schema::devices::dsl::*;
            devices.filter(updated_at.lt(cutoff_date.naive_utc())).select(id).load(&mut self.connection)?
        };
        Ok(delete_subscriptions_for_device_ids(self, device_ids)?)
    }

    fn get_inactive_devices(&mut self, min_days: i64, max_days: i64, push_enabled: Option<bool>) -> Result<Vec<Device>, DatabaseError> {
        let min_days_cutoff = Utc::now() - Duration::days(min_days);
        let max_days_cutoff = Utc::now() - Duration::days(max_days);

        let filters = [DeviceFilter::CreatedBetween {
            start: max_days_cutoff.naive_utc(),
            end: min_days_cutoff.naive_utc(),
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
    use crate::{Database, DevicesRepository};

    #[tokio::test]
    async fn test_get_device_record_missing_row_is_not_found() {
        let error = Database::mock().run(|client| client.get_device_record("missing-device")).await.unwrap_err();

        assert!(error.is_not_found());
    }
}
