pub const METHOD_SESSION_PROPOSE: &str = "wc_sessionPropose";
pub const METHOD_SESSION_AUTHENTICATE: &str = "wc_sessionAuthenticate";
pub const METHOD_SESSION_SETTLE: &str = "wc_sessionSettle";
pub const METHOD_SESSION_REQUEST: &str = "wc_sessionRequest";
pub const METHOD_SESSION_UPDATE: &str = "wc_sessionUpdate";
pub const METHOD_SESSION_DELETE: &str = "wc_sessionDelete";
pub const METHOD_SESSION_PING: &str = "wc_sessionPing";
pub const METHOD_PAIRING_PING: &str = "wc_pairingPing";
pub const METHOD_PAIRING_DELETE: &str = "wc_pairingDelete";

const FIVE_MINUTES: u64 = 300;
const THIRTY_SECONDS: u64 = 30;
const ONE_DAY: u64 = 86_400;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Publish {
    ProposalRejection,
    SessionRequestResponse,
    SessionUpdate,
    SessionDelete,
    SessionDeleteResponse,
    SessionPingResponse,
    PairingPingResponse,
    PairingDeleteResponse,
    AuthenticationApproval,
    AuthenticationRejection,
}

impl Publish {
    pub fn tag(&self) -> u32 {
        match self {
            Self::ProposalRejection => 1120,
            Self::SessionRequestResponse => 1109,
            Self::SessionUpdate => 1104,
            Self::SessionDelete => 1112,
            Self::SessionDeleteResponse => 1113,
            Self::SessionPingResponse => 1115,
            Self::PairingPingResponse => 1003,
            Self::PairingDeleteResponse => 1001,
            Self::AuthenticationApproval => 1117,
            Self::AuthenticationRejection => 1118,
        }
    }

    pub fn ttl(&self) -> u64 {
        match self {
            Self::ProposalRejection | Self::SessionRequestResponse | Self::AuthenticationApproval | Self::AuthenticationRejection => FIVE_MINUTES,
            Self::SessionPingResponse | Self::PairingPingResponse => THIRTY_SECONDS,
            Self::SessionUpdate | Self::SessionDelete | Self::SessionDeleteResponse | Self::PairingDeleteResponse => ONE_DAY,
        }
    }
}
