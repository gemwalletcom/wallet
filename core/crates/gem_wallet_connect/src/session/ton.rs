use std::collections::HashMap;

use gem_ton::{Address as TonAddress, signer::WalletV4R2};
use primitives::{Account, Chain};

const GET_PUBLIC_KEY_PROPERTY: &str = "ton_getPublicKey";
const GET_STATE_INIT_PROPERTY: &str = "ton_getStateInit";

pub(super) fn ton_session_properties(properties: &mut HashMap<String, String>, accounts: &[Account]) {
    let Some((public_key, state_init)) = accounts.iter().find(|account| account.chain == Chain::Ton).and_then(ton_account_properties) else {
        return;
    };
    properties.insert(GET_PUBLIC_KEY_PROPERTY.to_string(), public_key);
    properties.insert(GET_STATE_INIT_PROPERTY.to_string(), state_init);
}

fn ton_account_properties(account: &Account) -> Option<(String, String)> {
    let public_key: [u8; 32] = hex::decode(account.extended_public_key.as_deref()?).ok()?.try_into().ok()?;
    let wallet = WalletV4R2::new(public_key).ok()?;
    if TonAddress::parse(&account.address).ok()? != wallet.address {
        return None;
    }
    Some((hex::encode(public_key), wallet.state_init_base64().ok()?))
}

#[cfg(test)]
mod tests {
    use gem_ton::tvm::BagOfCells;

    use super::*;
    use crate::{
        session::config_session_properties,
        testkit::{TEST_TON_ADDRESS, TEST_TON_PUBLIC_KEY, mock_ton_account},
    };

    #[test]
    fn test_config_session_properties_ton() {
        let properties = HashMap::from([
            (GET_PUBLIC_KEY_PROPERTY.to_string(), "invalid".to_string()),
            (GET_STATE_INIT_PROPERTY.to_string(), "invalid".to_string()),
        ]);
        let result = config_session_properties(properties, &[Chain::Ton], &[mock_ton_account()]);
        assert_eq!(result.get(GET_PUBLIC_KEY_PROPERTY).unwrap(), TEST_TON_PUBLIC_KEY);

        let state_init = BagOfCells::parse_base64_root(result.get(GET_STATE_INIT_PROPERTY).unwrap()).unwrap();
        assert_eq!(state_init.hash, TonAddress::parse(TEST_TON_ADDRESS).unwrap().hash_part().to_owned());

        let account = Account {
            extended_public_key: None,
            ..mock_ton_account()
        };
        let result = config_session_properties(HashMap::new(), &[Chain::Ton], &[account]);
        assert_eq!(result.get(GET_PUBLIC_KEY_PROPERTY), None);
        assert_eq!(result.get(GET_STATE_INIT_PROPERTY), None);

        let account = Account {
            address: "UQBY1cVPu4SIr36q0M3HWcqPb_efyVVRBsEzmwN-wKQDR6zg".to_string(),
            ..mock_ton_account()
        };
        let result = config_session_properties(HashMap::new(), &[Chain::Ton], &[account]);
        assert_eq!(result.get(GET_PUBLIC_KEY_PROPERTY), None);
        assert_eq!(result.get(GET_STATE_INIT_PROPERTY), None);

        let result = config_session_properties(HashMap::new(), &[Chain::Ethereum], &[mock_ton_account()]);
        assert_eq!(result.get(GET_PUBLIC_KEY_PROPERTY), None);
        assert_eq!(result.get(GET_STATE_INIT_PROPERTY), None);
    }
}
