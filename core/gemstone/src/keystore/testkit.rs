use primitives::Chain;
use primitives::testkit::ABANDON_PHRASE;

use super::GemImportType;

pub fn mock_phrase_words() -> Vec<String> {
    ABANDON_PHRASE.split_whitespace().map(|word| word.to_string()).collect()
}

impl GemImportType {
    pub fn mock_private_key() -> Self {
        Self::PrivateKey {
            value: "0x30df0ffc2b43717f4653c2a1e827e9dfb3d9364e019cc60092496cd4997d5d6e".to_string(),
            chain: Chain::Ethereum,
        }
    }

    pub fn mock_multicoin_phrase(chains: Vec<Chain>) -> Self {
        Self::MulticoinPhrase {
            words: mock_phrase_words(),
            chains,
        }
    }

    pub fn mock_single_phrase() -> Self {
        Self::SinglePhrase {
            words: mock_phrase_words(),
            chain: Chain::Solana,
        }
    }
}
