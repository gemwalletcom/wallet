#[cfg(unix)]
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Read, Take};
#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::Path;

use crate::KeystoreError;

#[cfg(unix)]
const OWNER_READ_WRITE: u32 = 0o600;
#[cfg(not(unix))]
const UNSUPPORTED_SECRET_FILES: &str = "owner-only secret files";

pub(crate) fn read_capped(path: &Path, cap: usize) -> Result<Vec<u8>, KeystoreError> {
    let mut file = File::open(path)?;
    let mut bytes = Vec::new();
    let mut capped: Take<&mut File> = std::io::Read::by_ref(&mut file).take((cap + 1) as u64);
    capped.read_to_end(&mut bytes)?;
    if bytes.len() > cap {
        return Err(KeystoreError::corrupt_file("file too large"));
    }
    Ok(bytes)
}

#[cfg(unix)]
pub(super) fn new_secret_file_options() -> Result<OpenOptions, KeystoreError> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true).mode(OWNER_READ_WRITE);
    Ok(options)
}

#[cfg(not(unix))]
pub(super) fn new_secret_file_options() -> Result<OpenOptions, KeystoreError> {
    Err(KeystoreError::unsupported(UNSUPPORTED_SECRET_FILES))
}

#[cfg(unix)]
pub(super) fn set_owner_read_write(path: &Path) -> Result<(), KeystoreError> {
    fs::set_permissions(path, fs::Permissions::from_mode(OWNER_READ_WRITE))?;
    Ok(())
}

#[cfg(not(unix))]
pub(super) fn set_owner_read_write(_path: &Path) -> Result<(), KeystoreError> {
    Err(KeystoreError::unsupported(UNSUPPORTED_SECRET_FILES))
}

pub(super) fn sync_directory(path: &Path) -> Result<(), KeystoreError> {
    File::open(path)?.sync_all()?;
    Ok(())
}
