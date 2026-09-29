use gem_client::Target;

pub const SWAPS_LIMIT: usize = 500;
const SWAP_STATES: [&str; 7] = ["waiting", "receiving", "swapping", "sending", "sent", "completed", "failed"];

#[derive(Clone, Debug)]
pub enum BrokerTarget {
    Assets,
    Quotes,
    Rpc,
    Swaps { api_key: String, offset: u64 },
}

impl Target for BrokerTarget {
    fn path(&self) -> String {
        match self {
            Self::Assets => "/assets".to_string(),
            Self::Quotes => "/quotes-native".to_string(),
            Self::Rpc => "/rpc".to_string(),
            Self::Swaps { api_key, offset } => {
                let states = SWAP_STATES.iter().map(|state| format!("&status={state}")).collect::<String>();
                let api_key = if api_key.is_empty() { String::new() } else { format!("&apiKey={api_key}") };
                format!("/swaps?sort=ascending&limit={SWAPS_LIMIT}&offset={offset}{states}{api_key}")
            }
        }
    }
}
