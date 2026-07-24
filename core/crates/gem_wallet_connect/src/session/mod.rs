mod sui;
mod ton;
mod tron;

use std::collections::HashMap;

use primitives::{Account, Chain};

use self::{sui::sui_session_properties, ton::ton_session_properties, tron::tron_session_properties};

pub fn config_session_properties(mut properties: HashMap<String, String>, chains: &[Chain], accounts: &[Account]) -> HashMap<String, String> {
    for chain in chains {
        match chain {
            Chain::Tron => tron_session_properties(&mut properties),
            Chain::Ton => ton_session_properties(&mut properties, accounts),
            Chain::Sui => sui_session_properties(&mut properties, accounts),
            _ => {}
        }
    }
    properties
}
