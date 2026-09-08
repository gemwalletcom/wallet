use primitives::Chain;

pub fn chain_id(chain: Chain) -> Option<&'static str> {
    match chain {
        Chain::SmartChain => Some("bsc"),
        Chain::AvalancheC => Some("avalanche"),
        Chain::SeiEvm => Some("sei"),
        Chain::Hyperliquid => Some("hyperevm"),
        Chain::World => Some("worldchain"),
        Chain::ZkSync => Some("zksync"),
        Chain::Ethereum => Some("ethereum"),
        Chain::Solana => Some("solana"),
        Chain::Polygon => Some("polygon"),
        Chain::Arbitrum => Some("arbitrum"),
        Chain::Optimism => Some("optimism"),
        Chain::Base => Some("base"),
        Chain::Ton => Some("ton"),
        Chain::Tron => Some("tron"),
        Chain::Sui => Some("sui"),
        Chain::OpBNB => Some("opbnb"),
        Chain::Fantom => Some("fantom"),
        Chain::Gnosis => Some("gnosischain"),
        Chain::Manta => Some("manta"),
        Chain::Blast => Some("blast"),
        Chain::Linea => Some("linea"),
        Chain::Mantle => Some("mantle"),
        Chain::Celo => Some("celo"),
        Chain::Near => Some("near"),
        Chain::Sonic => Some("sonic"),
        Chain::Abstract => Some("abstract"),
        Chain::Berachain => Some("berachain"),
        Chain::Ink => Some("ink"),
        Chain::Unichain => Some("unichain"),
        Chain::Plasma => Some("plasma"),
        Chain::Monad => Some("monad"),
        Chain::XLayer => Some("xlayer"),
        Chain::Robinhood => Some("robinhood"),
        Chain::Stable => Some("stable"),
        Chain::Tempo => Some("tempo"),
        Chain::Bitcoin
        | Chain::BitcoinCash
        | Chain::Litecoin
        | Chain::Thorchain
        | Chain::Mayachain
        | Chain::Cosmos
        | Chain::Osmosis
        | Chain::Doge
        | Chain::Zcash
        | Chain::Aptos
        | Chain::Xrp
        | Chain::Celestia
        | Chain::Injective
        | Chain::Sei
        | Chain::Noble
        | Chain::Stellar
        | Chain::Algorand
        | Chain::Polkadot
        | Chain::Cardano
        | Chain::HyperCore => None,
    }
}
