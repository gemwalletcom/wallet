use gem_encoding::{decode_base64, encode_base64};
use signer::Ed25519KeyPair;

use crate::{Pubkey, VersionedTransaction, VersionedTransactionExt, decode_transaction, testkit::test_wallet_pubkey};

pub const TEST_RECIPIENT: &str = "EN2sCsJ1WDV8UFqsiTXHcUPUxQ4juE71eCknHYYMifkd";
pub const TEST_SENDER_TOKEN_ADDRESS: &str = "HEeranxp3y7kVQKVSLdZW1rUmnbs7bAtUTMu8o88Jash";

pub fn transaction_with_wallet_fee_payer(encoded_base64: &str) -> String {
    let mut transaction = decode_transaction(encoded_base64).unwrap();
    transaction.account_keys_mut()[0] = test_wallet_pubkey();
    encode_base64(&transaction.serialize().unwrap())
}

pub fn sender_address_for_key(private_key: &[u8]) -> String {
    let key_pair = Ed25519KeyPair::from_private_key(private_key).unwrap();
    bs58::encode(key_pair.public_key_bytes).into_string()
}

pub fn private_key_base58(value: &str) -> Vec<u8> {
    bs58::decode(value).into_vec().unwrap()
}

pub fn base58_transaction(encoded_base64: &str) -> String {
    bs58::encode(decode_base64(encoded_base64).unwrap()).into_string()
}

pub fn program_id(transaction: &VersionedTransaction, index: usize) -> String {
    let instruction = &transaction.instructions()[index];
    transaction.account_keys()[instruction.program_id_index as usize].to_base58()
}

pub fn account_key(transaction: &VersionedTransaction, instruction_index: usize, account_index: usize) -> Pubkey {
    let instruction = &transaction.instructions()[instruction_index];
    transaction.account_keys()[instruction.accounts[account_index] as usize]
}

pub const SINGLE_SIG_TX: &str = "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACAAQAIE4X7qT7inGBPqFijUWiMASkIQer7GcY6cKR108e8O++fF6bw89YHC7acs3m1YUIhVlGt8Lsh5HSIZozEPbcak9lEkZ+TiEMZdELfvcljcnBtv3Cf2z6oSYOBlVK0qYEJiVPvd6ryg3kqPlsEMcw3Fwx5sgoudurpYDKruhuxfayEdR478uf/smdZykpYhZIZWgIVX3IpDQW2WlWxw1AX3C53BHo4HDkVOPejukK6/oQdRT8m1S5xpmRD9q8e3XSK/Xu3TqNZNjLp6OSRg8r3sOv1e+QztSj31QG3tKRlT5zLqo0WQ1mJ0Hxkrw69L1dQmgMqgZd20xmPPvg53n23dsfNG6CPj/KUTsrekiJQc2Hvabji48RBleABXKq2lBKpj89u0FRUiC4li6CDgDgrZl6r6UV4hTXUW6fWb5yNguwg6dRIiwf+OZsakVXlghtpfUMBbAo8Tzu8oq+0HQFjMFcAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMGRm/lIRcy/+ytunLDm+e8jOW7xfcSayxDmzpAAAAABHnVW/IxwG7udMVuzmgVB/2xst6j9I5RArHNola8E48G3fbh12Whk9nL4UbO63msHLSF7V9bN5E6jPWFfv8AqYyXJY9OJInxuz0QKRSODYMLWhOZ2v8QhASOe9jb6fhZrBrj0IfykjcGJUj3DEwErsKplWlJhufLtGdSBiHThjC0P/on9df2SnTAmx8pWHneSwmrNt/J3VFLMhqns4zl6Mb6evO+2606PWXzaqvJdDGxu+TC0vbg5HymAgNFL11hhfF5aGC8uN+dIBDrxV3vHwZYs2RmFKm87C1HtO3e+NIHDAAFAsBcFQAPBgAJACwLDgEBCwIACQwCAAAAgJaYAAAAAAAOAQkBEQ8GAAYAEgsOAQENQg4QAAkFCgYsEgcNEQ0qKRYZCgUXGBArDg0nDi0uEAUhIgQfICYOHiUdGhsoHB4eHh4eHgQDECMOEBQDFQoTCAIBJDLBIJszQdacgQUEAAAAEgA8AAQDKAACB2QCAxEBZAMEgJaYAAAAAACBqQkAAAAAADIAMg4DCQAAAQkEB4tUBJUod+xdJHClaAbwfY0KcsPyS6puvinwAKI7S1cD9e7vBh7xJzZ/XBUMq/0+5x74cLXUfrm4RcW7GMg1vW5lr144gPYI6KKjBMO8u74DwL/CbfplXQ5y4uTDJAQIjShVPQLz8v+me8U9jJd68IPKjLkFvL08uLoAzNqZz4glI8qy5jvrpHYo8Vzh6SmqRgs5a0H2AAOFaZwE6uzp5wNl7es=";
