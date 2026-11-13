use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use rusqlite::{Connection, params};

use crate::error::LightningError;

const BUSY_TIMEOUT: Duration = Duration::from_secs(5);
const SCHEMA: &str = "
    CREATE TABLE IF NOT EXISTS gem_pending_sends (
        transfer_id TEXT PRIMARY KEY,
        invoice TEXT NOT NULL,
        amount_sats INTEGER,
        expires_at INTEGER NOT NULL
    );
";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingSend {
    pub transfer_id: String,
    pub invoice: String,
    pub amount_sats: Option<u64>,
    pub expires_at: u64,
}

pub struct WalletDatabase {
    connection: Mutex<Connection>,
}

impl WalletDatabase {
    pub fn open(path: &Path) -> Result<Self, LightningError> {
        let connection = Connection::open(path)?;
        connection.busy_timeout(BUSY_TIMEOUT)?;
        connection.execute_batch(SCHEMA)?;
        Ok(Self { connection: Mutex::new(connection) })
    }

    pub fn add_pending_send(&self, send: &PendingSend) -> Result<(), LightningError> {
        self.connection()?.execute(
            "INSERT INTO gem_pending_sends (transfer_id, invoice, amount_sats, expires_at) VALUES (?1, ?2, ?3, ?4)",
            params![send.transfer_id, send.invoice, send.amount_sats, send.expires_at],
        )?;
        Ok(())
    }

    pub fn get_pending_sends(&self) -> Result<Vec<PendingSend>, LightningError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare("SELECT transfer_id, invoice, amount_sats, expires_at FROM gem_pending_sends ORDER BY transfer_id")?;
        let rows = statement.query_map([], |row| {
            Ok(PendingSend {
                transfer_id: row.get(0)?,
                invoice: row.get(1)?,
                amount_sats: row.get(2)?,
                expires_at: row.get(3)?,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn delete_pending_send(&self, transfer_id: &str) -> Result<(), LightningError> {
        self.connection()?.execute("DELETE FROM gem_pending_sends WHERE transfer_id = ?1", params![transfer_id])?;
        Ok(())
    }

    fn connection(&self) -> Result<MutexGuard<'_, Connection>, LightningError> {
        self.connection.lock().map_err(|error| LightningError::Storage(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pending_sends() {
        let dir = primitives::testkit::temp_dir::TempDir::new().unwrap();
        let database = WalletDatabase::open(&dir.path().join("wallet.sqlite")).unwrap();
        let first = PendingSend {
            transfer_id: "0199f0a0-0000-7000-8000-000000000001".to_string(),
            invoice: "lnbc1".to_string(),
            amount_sats: None,
            expires_at: 1_800_000_000,
        };
        let second = PendingSend {
            transfer_id: "0199f0a0-0000-7000-8000-000000000002".to_string(),
            amount_sats: Some(2000),
            ..first.clone()
        };

        database.add_pending_send(&first).unwrap();
        database.add_pending_send(&second).unwrap();
        assert_eq!(database.get_pending_sends().unwrap(), vec![first.clone(), second.clone()]);
        assert!(database.add_pending_send(&first).is_err());

        database.delete_pending_send(&first.transfer_id).unwrap();
        assert_eq!(WalletDatabase::open(&dir.path().join("wallet.sqlite")).unwrap().get_pending_sends().unwrap(), vec![second]);
    }
}
