use std::hint::black_box;
use std::time::{Duration, Instant};

use gem_keystore::{FileKeystore, Mnemonic};
use tempfile::TempDir;

const ITERATIONS: usize = 5;

fn main() {
    let dir = TempDir::new().unwrap();
    let keystore = FileKeystore::open(dir.path().to_path_buf()).unwrap();
    let phrase = Mnemonic::generate(12).unwrap().join(" ");
    let password = gem_crypto::random::bytes::<32>().unwrap();

    let mut keystore_id = keystore.import_mnemonic(&phrase, &password, None).unwrap().keystore_id;
    let mut encrypt = Vec::new();
    for _ in 0..ITERATIONS {
        keystore.delete(&keystore_id).unwrap();
        let start = Instant::now();
        keystore_id = keystore.import_mnemonic(black_box(&phrase), black_box(&password), None).unwrap().keystore_id;
        encrypt.push(start.elapsed());
    }

    keystore.decrypt_mnemonic(&keystore_id, &password).unwrap();
    let mut decrypt = Vec::new();
    for _ in 0..ITERATIONS {
        let start = Instant::now();
        black_box(keystore.decrypt_mnemonic(&keystore_id, black_box(&password)).unwrap());
        decrypt.push(start.elapsed());
    }

    println!("keystore_v4 encrypt(import_mnemonic) median: {:?}", median(&mut encrypt));
    println!("keystore_v4 decrypt(decrypt_mnemonic) median: {:?}", median(&mut decrypt));
}

fn median(durations: &mut [Duration]) -> Duration {
    durations.sort();
    durations[durations.len() / 2]
}
