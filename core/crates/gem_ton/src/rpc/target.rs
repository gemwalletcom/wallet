use std::collections::HashMap;

use gem_client::{Target, build_path_with_query};

use crate::models::{TraceByAddressQuery, TraceByBlockQuery, TraceByMessageQuery, TraceByTransactionQuery};

const ACTIONS_VERSION_HEADER: &str = "X-Actions-Version";
const ACTIONS_VERSION: &str = "5";
const JETTON_WALLETS_LIMIT: usize = 100;
const NFT_ITEMS_LIMIT: usize = 1000;

#[derive(Clone, Debug)]
pub enum TonCenterTarget {
    GetMasterchainInfo,
    GetDnsRecords { domain: String },
    GetJettonMasters { address: String },
    GetAddressBalance { address: String },
    GetWalletInformation { address: String },
    SendBoc,
    EmulateTonConnect,
    RunGetMethod,
    GetTracesByMessage { query: TraceByMessageQuery },
    GetTracesByTransaction { query: TraceByTransactionQuery },
    GetTracesByBlock { query: TraceByBlockQuery },
    GetTracesByAddress { query: TraceByAddressQuery },
    GetJettonWallets { owner: String },
    GetNftItemsByOwner { owner: String },
    GetNftItem { address: String },
    GetNftCollection { address: String },
}

impl Target for TonCenterTarget {
    fn path(&self) -> String {
        match self {
            Self::GetMasterchainInfo => "/api/v3/masterchainInfo".to_string(),
            Self::GetDnsRecords { domain } => format!("/api/v3/dns/records?domain={domain}&limit=1"),
            Self::GetJettonMasters { address } => format!("/api/v3/jetton/masters?address={address}"),
            Self::GetAddressBalance { address } => format!("/api/v2/getAddressBalance?address={address}"),
            Self::GetWalletInformation { address } => format!("/api/v2/getWalletInformation?address={address}"),
            Self::SendBoc => "/api/v2/sendBocReturnHash".to_string(),
            Self::EmulateTonConnect => "/api/emulate/v1/emulateTonConnect".to_string(),
            Self::RunGetMethod => "/api/v2/runGetMethod".to_string(),
            Self::GetTracesByMessage { query } => build_path_with_query("/api/v3/traces", query),
            Self::GetTracesByTransaction { query } => build_path_with_query("/api/v3/traces", query),
            Self::GetTracesByBlock { query } => build_path_with_query("/api/v3/traces", query),
            Self::GetTracesByAddress { query } => build_path_with_query("/api/v3/traces", query),
            Self::GetJettonWallets { owner } => format!("/api/v3/jetton/wallets?owner_address={owner}&limit={JETTON_WALLETS_LIMIT}&offset=0"),
            Self::GetNftItemsByOwner { owner } => format!("/api/v3/nft/items?owner_address={owner}&limit={NFT_ITEMS_LIMIT}&offset=0"),
            Self::GetNftItem { address } => format!("/api/v3/nft/items?address={address}"),
            Self::GetNftCollection { address } => format!("/api/v3/nft/collections?collection_address={address}"),
        }
    }

    fn headers(&self) -> HashMap<String, String> {
        match self {
            Self::EmulateTonConnect => HashMap::from([(ACTIONS_VERSION_HEADER.to_string(), ACTIONS_VERSION.to_string())]),
            _ => HashMap::new(),
        }
    }
}
