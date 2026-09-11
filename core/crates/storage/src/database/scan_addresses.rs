use crate::schema::scan_addresses::dsl::*;
use crate::{DatabaseClient, models::*};
use diesel::{prelude::*, upsert::excluded};

pub(crate) trait ScanAddressesStore {
    fn get_scan_addresses_by_addresses(&mut self, addresses: Vec<String>) -> Result<Vec<ScanAddressRow>, diesel::result::Error>;
    fn add_scan_addresses(&mut self, values: Vec<NewScanAddressRow>) -> Result<usize, diesel::result::Error>;
    fn upsert_scan_addresses(&mut self, values: Vec<NewScanAddressRow>) -> Result<usize, diesel::result::Error>;
}

impl ScanAddressesStore for DatabaseClient {
    fn get_scan_addresses_by_addresses(&mut self, addresses: Vec<String>) -> Result<Vec<ScanAddressRow>, diesel::result::Error> {
        scan_addresses
            .filter(address.eq_any(addresses))
            .order((address.asc(), id.asc()))
            .select(ScanAddressRow::as_select())
            .load(&mut self.connection)
    }

    fn add_scan_addresses(&mut self, values: Vec<NewScanAddressRow>) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(scan_addresses).values(values).on_conflict_do_nothing().execute(&mut self.connection)
    }

    fn upsert_scan_addresses(&mut self, values: Vec<NewScanAddressRow>) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(scan_addresses)
            .values(values)
            .on_conflict((chain, address))
            .do_update()
            .set(name.eq(excluded(name)))
            .execute(&mut self.connection)
    }
}
