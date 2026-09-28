use chrono::{Duration, NaiveDateTime, Utc};
use diesel::{prelude::*, upsert::excluded};
use primitives::Device;

use crate::models::{DeviceRow, UpdateDeviceRow};
use crate::repositories::wallets_repository::delete_subscriptions_for_device_ids;
use crate::{DatabaseClient, DatabaseError, DieselResultExt};

#[derive(Debug, Clone)]
pub enum DeviceFieldUpdate {
    IsPushEnabled(bool),
    IsPriceAlertsEnabled(bool),
}

#[derive(Debug, Clone)]
enum DeviceFilter {
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

pub trait DevicesRepository {
    fn add_device(&mut self, device: Device) -> Result<Device, DatabaseError>;
    fn get_device(&mut self, device_id: &str) -> Result<Device, DatabaseError>;
    fn get_device_record(&mut self, device_id: &str) -> Result<DeviceRecord, DatabaseError>;
    fn get_device_exist(&mut self, device_id: &str) -> Result<bool, DatabaseError>;
    fn get_device_row_id(&mut self, device_id: &str) -> Result<i32, DatabaseError>;
    fn update_device(&mut self, device: Device) -> Result<Device, DatabaseError>;
    fn update_device_fields(&mut self, device_ids: Vec<String>, updates: Vec<DeviceFieldUpdate>) -> Result<usize, DatabaseError>;
    fn delete_devices_subscriptions_after_days(&mut self, days: i64) -> Result<usize, DatabaseError>;
    fn devices_inactive_days(&mut self, min_days: i64, max_days: i64, push_enabled: Option<bool>) -> Result<Vec<Device>, DatabaseError>;
}

pub(crate) fn device_row(client: &mut DatabaseClient, device_id_value: &str) -> Result<DeviceRow, diesel::result::Error> {
    use crate::schema::devices::dsl::*;
    devices.filter(device_id.eq(device_id_value)).select(DeviceRow::as_select()).first(&mut client.connection)
}

impl DevicesRepository for DatabaseClient {
    fn add_device(&mut self, device: Device) -> Result<Device, DatabaseError> {
        use crate::schema::devices::dsl::*;
        let device = UpdateDeviceRow::from_primitive(device);
        Ok(diesel::insert_into(devices)
            .values(&device)
            .on_conflict(device_id)
            .do_update()
            .set((device_id.eq(excluded(device_id)),))
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
        let device_id_value = device.device_id.clone();
        use crate::schema::devices::dsl::*;
        Ok(diesel::update(devices)
            .filter(device_id.eq(device_id_value.clone()))
            .set(device)
            .returning(DeviceRow::as_returning())
            .get_result(&mut self.connection)
            .or_not_found(device_id_value)?
            .as_primitive())
    }

    fn update_device_fields(&mut self, device_ids: Vec<String>, updates: Vec<DeviceFieldUpdate>) -> Result<usize, DatabaseError> {
        use crate::schema::devices::dsl::*;

        if updates.is_empty() || device_ids.is_empty() {
            return Ok(0);
        }

        let mut total_updated = 0;
        for update in updates {
            let target = devices.filter(device_id.eq_any(&device_ids));
            let updated = match update {
                DeviceFieldUpdate::IsPushEnabled(value) => diesel::update(target).set(is_push_enabled.eq(value)).execute(&mut self.connection)?,
                DeviceFieldUpdate::IsPriceAlertsEnabled(value) => diesel::update(target).set(is_price_alerts_enabled.eq(value)).execute(&mut self.connection)?,
            };
            total_updated += updated;
        }

        Ok(total_updated)
    }

    fn delete_devices_subscriptions_after_days(&mut self, days: i64) -> Result<usize, DatabaseError> {
        let cutoff_date = Utc::now() - Duration::days(days);
        let device_ids: Vec<i32> = {
            use crate::schema::devices::dsl::*;
            devices.filter(updated_at.lt(cutoff_date.naive_utc())).select(id).load(&mut self.connection)?
        };
        Ok(delete_subscriptions_for_device_ids(self, device_ids)?)
    }

    fn devices_inactive_days(&mut self, min_days: i64, max_days: i64, push_enabled: Option<bool>) -> Result<Vec<Device>, DatabaseError> {
        let min_days_cutoff = Utc::now() - Duration::days(min_days);
        let max_days_cutoff = Utc::now() - Duration::days(max_days);

        let mut filters = vec![DeviceFilter::CreatedBetween {
            start: max_days_cutoff.naive_utc(),
            end: min_days_cutoff.naive_utc(),
        }];

        if let Some(enabled) = push_enabled {
            filters.push(DeviceFilter::IsPushEnabled(enabled));
        }

        use crate::schema::devices::dsl::*;

        let mut query = devices.into_boxed();

        for filter in filters {
            match filter {
                DeviceFilter::IsPushEnabled(enabled) => {
                    query = query.filter(is_push_enabled.eq(enabled));
                }
                DeviceFilter::CreatedBetween { start, end } => {
                    query = query.filter(
                        created_at
                            .between(start, end)
                            .and(diesel::dsl::sql::<diesel::sql_types::Bool>("DATE_TRUNC('hour', updated_at) = DATE_TRUNC('hour', created_at)")),
                    );
                }
            }
        }

        Ok(query.select(DeviceRow::as_select()).load(&mut self.connection)?.into_iter().map(|x| x.as_primitive()).collect())
    }
}
