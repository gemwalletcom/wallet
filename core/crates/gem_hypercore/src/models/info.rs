use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum InfoRequest {
    ValidatorSummaries,
    Delegations {
        user: String,
    },
    SpotClearinghouseState {
        user: String,
    },
    DelegatorSummary {
        user: String,
    },
    UserFillsByTime {
        user: String,
        start_time: i64,
    },
    ClearinghouseState {
        user: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        dex: Option<String>,
    },
    MetaAndAssetCtxs {
        #[serde(skip_serializing_if = "Option::is_none")]
        dex: Option<String>,
    },
    PerpDexs,
    SpotMeta,
    L2Book {
        coin: String,
    },
    CandleSnapshot {
        req: CandleSnapshotRequest,
    },
    UserAbstraction {
        user: String,
    },
    Referral {
        user: String,
    },
    ExtraAgents {
        user: String,
    },
    MaxBuilderFee {
        user: String,
        builder: String,
    },
    UserFees {
        user: String,
    },
    UserNonFundingLedgerUpdates {
        user: String,
        start_time: i64,
    },
    DelegatorHistory {
        user: String,
    },
    FrontendOpenOrders {
        user: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        dex: Option<String>,
    },
    Portfolio {
        user: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        dex: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CandleSnapshotRequest {
    pub coin: String,
    pub interval: String,
    pub start_time: i64,
    pub end_time: i64,
}
