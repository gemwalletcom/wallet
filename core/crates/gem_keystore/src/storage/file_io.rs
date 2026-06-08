use std::fs::{self, File, OpenOptions};
use std::io::{Read, Take};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::Path;

use crate::KeystoreError;

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

pub(super) fn new_secret_file_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    set_secret_file_mode(&mut options);
    options
}

fn set_secret_file_mode(options: &mut OpenOptions) {
    options.mode(0o600);
}

pub(super) fn set_owner_read_write(path: &Path) -> Result<(), KeystoreError> {
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}

pub(super) fn sync_directory(path: &Path) -> Result<(), KeystoreError> {
    File::open(path)?.sync_all()?;
    Ok(())
}
