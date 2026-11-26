use primitives::ChainType;

#[derive(Clone, Copy)]
pub(super) enum DataFormat {
    EvmCall,
    SolanaTransaction,
    TonMessages,
    Unread,
}

impl DataFormat {
    pub(super) fn of(chain_type: ChainType) -> Self {
        match chain_type {
            ChainType::Ethereum | ChainType::Tron => Self::EvmCall,
            ChainType::Solana => Self::SolanaTransaction,
            ChainType::Ton => Self::TonMessages,
            ChainType::Bitcoin | ChainType::Cosmos | ChainType::Aptos | ChainType::Sui | ChainType::Xrp | ChainType::Near | ChainType::Stellar | ChainType::Algorand | ChainType::Polkadot | ChainType::Cardano | ChainType::HyperCore => {
                Self::Unread
            }
        }
    }
}
