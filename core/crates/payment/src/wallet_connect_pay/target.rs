use gem_client::{Target, build_path_with_query};

const QUERY_INCLUDE_PAYMENT_INFO: &str = "includePaymentInfo";

#[derive(Clone, Debug)]
pub(super) enum WalletConnectPayTarget {
    Options { payment_id: String },
    Fetch { payment_id: String },
    Confirm { payment_id: String },
}

impl Target for WalletConnectPayTarget {
    fn path(&self) -> String {
        match self {
            Self::Options { payment_id } => build_path_with_query(&payment_path(payment_id, "options"), &[(QUERY_INCLUDE_PAYMENT_INFO, "true")]),
            Self::Fetch { payment_id } => payment_path(payment_id, "fetch"),
            Self::Confirm { payment_id } => payment_path(payment_id, "confirm"),
        }
    }
}

fn payment_path(payment_id: &str, action: &str) -> String {
    format!("/v1/gateway/payment/{payment_id}/{action}")
}
