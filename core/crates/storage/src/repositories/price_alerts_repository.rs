use std::collections::{HashMap, HashSet};
use std::time::Duration;

use chrono::NaiveDateTime;
use diesel::prelude::*;
use primitives::{AssetId, Device, DevicePriceAlert, PriceAlert, PriceAlerts, PriceData};

use crate::models::{DeviceRow, PriceAlertRow};
use crate::repositories::devices_repository::device_row;
use crate::repositories::prices_repository::PricesRepository;
use crate::{DatabaseClient, DatabaseError, DieselResultExt};

pub trait PriceAlertsRepository {
    fn get_price_alerts(&mut self, after_notified_at: NaiveDateTime, max_age: Duration) -> Result<Vec<(PriceAlert, PriceData, Device)>, DatabaseError>;
    fn get_price_alerts_for_device_id(&mut self, device_id: &str, asset_id: Option<&AssetId>) -> Result<Vec<DevicePriceAlert>, DatabaseError>;
    fn count_price_alerts_for_device_id(&mut self, device_id: i32) -> Result<i64, DatabaseError>;
    fn add_price_alerts(&mut self, device_id: &str, price_alerts: PriceAlerts) -> Result<usize, DatabaseError>;
    fn delete_price_alerts(&mut self, device_id: &str, ids: Vec<String>) -> Result<usize, DatabaseError>;
    fn update_price_alerts_set_notified_at(&mut self, ids: Vec<String>, last_notified_at: NaiveDateTime) -> Result<usize, DatabaseError>;
}

impl PriceAlertsRepository for DatabaseClient {
    fn get_price_alerts(&mut self, after_notified_at: NaiveDateTime, max_age: Duration) -> Result<Vec<(PriceAlert, PriceData, Device)>, DatabaseError> {
        let alerts: Vec<(PriceAlertRow, DeviceRow)> = {
            use crate::schema::devices;
            use crate::schema::price_alerts::dsl::*;

            price_alerts
                .filter((price_direction.is_not_null().and(last_notified_at.is_null())).or(price_direction.is_null().and(last_notified_at.lt(after_notified_at).or(last_notified_at.is_null()))))
                .inner_join(devices::table.on(device_id.eq(devices::id)))
                .filter(devices::is_push_enabled.eq(true).and(devices::is_price_alerts_enabled.eq(true)).and(devices::token.ne("")))
                .select((PriceAlertRow::as_select(), DeviceRow::as_select()))
                .load(&mut self.connection)?
        };
        if alerts.is_empty() {
            return Ok(vec![]);
        }
        let asset_ids: Vec<AssetId> = alerts.iter().map(|(a, _)| a.asset_id.0.clone()).collect::<HashSet<_>>().into_iter().collect();
        let primary: HashMap<String, PriceData> = self.get_primary_prices(&asset_ids, max_age)?.into_iter().map(|(id, price)| (id.to_string(), price)).collect();
        Ok(alerts
            .into_iter()
            .filter_map(|(alert, device)| primary.get(&alert.asset_id.to_string()).map(|price| (alert.as_primitive(), price.clone(), device.as_primitive())))
            .collect())
    }

    fn get_price_alerts_for_device_id(&mut self, device_id_value: &str, asset_id_value: Option<&AssetId>) -> Result<Vec<DevicePriceAlert>, DatabaseError> {
        let asset_id_value = asset_id_value.map(ToString::to_string);
        use crate::schema::devices;
        use crate::schema::price_alerts::dsl::*;

        let mut query = price_alerts.inner_join(devices::table.on(device_id.eq(devices::id))).filter(devices::device_id.eq(device_id_value)).into_boxed();

        if let Some(asset_id_value) = asset_id_value {
            query = query.filter(asset_id.eq(asset_id_value));
        }

        let results: Vec<(PriceAlertRow, DeviceRow)> = query.select((PriceAlertRow::as_select(), DeviceRow::as_select())).load(&mut self.connection)?;
        Ok(results
            .into_iter()
            .map(|(alert, device)| DevicePriceAlert {
                device: device.as_primitive(),
                price_alert: alert.as_primitive(),
            })
            .collect())
    }

    fn count_price_alerts_for_device_id(&mut self, device_id_value: i32) -> Result<i64, DatabaseError> {
        use crate::schema::price_alerts::dsl::*;
        Ok(price_alerts.filter(device_id.eq(device_id_value)).count().get_result(&mut self.connection)?)
    }

    fn add_price_alerts(&mut self, device_id_value: &str, values: PriceAlerts) -> Result<usize, DatabaseError> {
        let device = device_row(self, device_id_value).or_not_found(device_id_value.to_string())?;
        let rows = values.into_iter().map(|x| PriceAlertRow::new_price_alert(x, device.id)).collect::<Vec<_>>();
        use crate::schema::price_alerts::dsl::*;
        Ok(diesel::insert_into(price_alerts)
            .values(rows)
            .on_conflict((device_id, identifier))
            .do_update()
            .set(last_notified_at.eq(Option::<NaiveDateTime>::None))
            .execute(&mut self.connection)?)
    }

    fn delete_price_alerts(&mut self, device_id_value: &str, ids: Vec<String>) -> Result<usize, DatabaseError> {
        let device = device_row(self, device_id_value).or_not_found(device_id_value.to_string())?;
        use crate::schema::price_alerts::dsl::*;
        Ok(diesel::delete(price_alerts.filter(device_id.eq(device.id).and(identifier.eq_any(ids)))).execute(&mut self.connection)?)
    }

    fn update_price_alerts_set_notified_at(&mut self, ids: Vec<String>, last_notified_at_value: NaiveDateTime) -> Result<usize, DatabaseError> {
        use crate::schema::price_alerts::dsl::*;
        Ok(diesel::update(price_alerts).filter(identifier.eq_any(&ids)).set(last_notified_at.eq(last_notified_at_value)).execute(&mut self.connection)?)
    }
}
