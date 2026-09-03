use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DnsRecordsResponse {
    pub records: Vec<DnsRecord>,
}

#[derive(Debug, Deserialize)]
pub struct DnsRecord {
    pub dns_wallet: Option<String>,
}
