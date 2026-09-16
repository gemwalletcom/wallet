pub fn mock_siwe_message(domain: &str, chain_id: u32) -> String {
    [
        &format!("{domain} wants you to sign in with your Ethereum account:"),
        "0x9EdcF9Ff72088DB8130C2512E5B4D3b5F34cEaF4",
        "",
        &format!("URI: https://{domain}"),
        "Version: 1",
        &format!("Chain ID: {chain_id}"),
        "Nonce: gmdhs9w9yfrl2kf2",
        "Issued At: 2026-03-06T01:56:42.927Z",
    ]
    .join("\n")
}

pub fn mock_siwe_message_hex(domain: &str, chain_id: u32) -> String {
    format!("0x{}", hex::encode(mock_siwe_message(domain, chain_id)))
}

pub fn mock_siwe_message_full() -> String {
    [
        "login.xyz wants you to sign in with your Ethereum account:",
        "0x6dD7802E6d44bE89a789C4bD60bD511B68F41c7c",
        "",
        "Sign in with Ethereum to the app.",
        "",
        "URI: https://login.xyz",
        "Version: 1",
        "Chain ID: 1",
        "Nonce: 8hK9pX32",
        "Issued At: 2024-04-01T12:00:00Z",
        "Expiration Time: 2024-04-02T12:00:00Z",
        "Not Before: 2024-04-01T11:00:00Z",
        "Request ID: abc-123",
        "Resources:",
        "- https://example.com/terms",
        "- https://example.com/privacy",
    ]
    .join("\n")
}
