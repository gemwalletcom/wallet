use crate::{SolanaNftStandard, SolanaTokenProgramId, TransactionLoadMetadata, stake_type::TronStakeData};

impl TransactionLoadMetadata {
    pub fn mock_osmosis() -> Self {
        TransactionLoadMetadata::Cosmos {
            account_number: 2_913_388,
            sequence: 10,
            chain_id: "osmosis-1".to_string(),
        }
    }

    pub fn mock_algorand(sequence: u64) -> Self {
        TransactionLoadMetadata::Algorand {
            sequence,
            block_hash: "SGO1GKSzyE7IEPItTxCByw9x8FmnrCDexi9/cOUJOiI=".to_string(),
            chain_id: "testnet-v1.0".to_string(),
        }
    }

    pub fn mock_evm(nonce: u64, chain_id: u64) -> Self {
        TransactionLoadMetadata::Evm {
            nonce,
            chain_id,
            contract_call: None,
        }
    }

    pub fn mock_tron() -> Self {
        Self::mock_tron_with_stake_data(TronStakeData::Votes { votes: vec![] })
    }

    pub fn mock_tron_with_stake_data(stake_data: TronStakeData) -> Self {
        TransactionLoadMetadata::Tron {
            block_number: 3_111_739,
            block_version: 3,
            block_timestamp: 1_539_295_479_000,
            transaction_tree_root: "64288c2db0641316762a99dbb02ef7c90f968b60f9f2e410835980614332f86d".to_string(),
            parent_hash: "00000000002f7b3af4f5f8b9e23a30c530f719f165b742e7358536b280eead2d".to_string(),
            witness_address: "415863f6091b8e71766da808b1dd3159790f61de7d".to_string(),
            stake_data,
        }
    }

    pub fn mock_tron_nile() -> Self {
        TransactionLoadMetadata::Tron {
            block_number: 34_395_330,
            block_version: 26,
            block_timestamp: 1_676_983_541_337,
            transaction_tree_root: "9b54db7f84bd19bbad9ff1fccef894c1aade6879450e9e9e2accec751eaa1f52".to_string(),
            parent_hash: "00000000020cd4c13a67497a3a433a3105bc5a73a041ee3da98407d5a2a2bf1b".to_string(),
            witness_address: "4150d3765e4e670727ebac9d5b598f74b75a3d54a7".to_string(),
            stake_data: TronStakeData::Votes { votes: vec![] },
        }
    }

    pub fn mock_ton(sequence: u64) -> Self {
        TransactionLoadMetadata::Ton {
            sender_token_address: None,
            recipient_token_address: None,
            sequence,
        }
    }

    pub fn mock_ton_jetton(sequence: u64, sender_token_address: &str) -> Self {
        TransactionLoadMetadata::Ton {
            sender_token_address: Some(sender_token_address.to_string()),
            recipient_token_address: None,
            sequence,
        }
    }

    pub fn mock_solana(block_hash: &str) -> Self {
        TransactionLoadMetadata::Solana {
            sender_token_address: None,
            recipient_token_address: None,
            token_program: None,
            nft: None,
            block_hash: block_hash.to_string(),
            references: vec![],
        }
    }

    pub fn mock_solana_transfer(
        sender_token_address: Option<&str>,
        recipient_token_address: Option<&str>,
        token_program: Option<SolanaTokenProgramId>,
        references: &[&str],
    ) -> Self {
        TransactionLoadMetadata::Solana {
            sender_token_address: sender_token_address.map(String::from),
            recipient_token_address: recipient_token_address.map(String::from),
            token_program,
            nft: None,
            block_hash: "11111111111111111111111111111111".to_string(),
            references: references.iter().map(|reference| reference.to_string()).collect(),
        }
    }

    pub fn mock_solana_nft(sender_token_address: &str, token_program: SolanaTokenProgramId, nft: SolanaNftStandard) -> Self {
        TransactionLoadMetadata::Solana {
            sender_token_address: Some(sender_token_address.to_string()),
            recipient_token_address: None,
            token_program: Some(token_program),
            nft: Some(nft),
            block_hash: "11111111111111111111111111111111".to_string(),
            references: vec![],
        }
    }

    pub fn mock_solana_core_nft(collection: Option<&str>) -> Self {
        TransactionLoadMetadata::Solana {
            sender_token_address: None,
            recipient_token_address: None,
            token_program: None,
            nft: Some(SolanaNftStandard::Core {
                collection: collection.map(String::from),
            }),
            block_hash: "11111111111111111111111111111111".to_string(),
            references: vec![],
        }
    }
}
