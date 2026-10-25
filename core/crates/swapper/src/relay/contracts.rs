use primitives::{
    Chain, ChainType,
    contract_constants::{SOLANA_JUPITER_PROGRAM_ID, SOLANA_RELAY_DEPOSITORY_PROGRAM_ID, SOLANA_RELAY_FORWARDER_PROGRAM_ID},
};

use crate::{Contracts, Deployment};

const DEPOSITORY: &str = "0x4cD00E387622C35bDDB9b4c962C136462338BC31";
const APPROVAL_PROXY: &str = "0xCcC88a9d1B4ED6b0EABA998850414b24f1c315bE";
const APPROVAL_PROXY_V3_2: &str = "0x5567954cD026CEbFE486062cb75848e0f1c12E58";
const ERC20_ROUTER: &str = "0xb92fe925DC43a0ECdE6c8b1a2709c170Ec4fFf4f";
const ERC20_ROUTER_V3_2: &str = "0x0729E2a4716d9f7dD17858d5d504426D228697f7";
const MANTLE_LINEA_DEPOSITORY: &str = "0x59916DA825D2D2eC1BF878D71c88826F6633ecca";
const MANTLE_LINEA_APPROVAL_PROXY: &str = "0x8754Bc615047de01228a7527B712806A71A8dc9a";
const MANTLE_LINEA_APPROVAL_PROXY_V3_2: &str = "0xE4DfBd5E6784dB1058124E2c87554C0591f0Fb7B";
const MANTLE_LINEA_ERC20_ROUTER: &str = "0x9EF6d3c2F60d7b9008D74CAb1FC0F899c957C819";
const MANTLE_LINEA_ERC20_ROUTER_V3_2: &str = "0x6328ea02d28131e58B28E55c6258258A30d90E05";
const TRON_DEPOSITORY: &str = "TXtEs6t2oUWQsNos7m68gbHdE9Q5n6x2oN";
const TON_DEPOSITORY: &str = "EQCrdGsDTqA2t6xRR4N6V4J705F7w_VQbUdHnofsh-8lVIPs";

const RECEIVERS: &[Deployment] = &[
    Deployment::new(Chain::Ethereum, "0xa5F565650890fBA1824Ee0F21EbBbF660a179934"),
    Deployment::new(Chain::Optimism, "0xa5F565650890fBA1824Ee0F21EbBbF660a179934"),
    Deployment::new(Chain::SmartChain, "0xa5F565650890fBA1824Ee0F21EbBbF660a179934"),
    Deployment::new(Chain::Gnosis, "0xa5F565650890fBA1824Ee0F21EbBbF660a179934"),
    Deployment::new(Chain::Unichain, "0x47eB64e17a6D2Fd559b608695E6d308cCED918Dd"),
    Deployment::new(Chain::Polygon, "0xa5F565650890fBA1824Ee0F21EbBbF660a179934"),
    Deployment::new(Chain::Monad, "0xf17902d51FdfF7Bf50AacC78d6bB399BaF88b479"),
    Deployment::new(Chain::Sonic, "0x7F4babd2C7D35221e72Ab67Ea72Cba99573A0089"),
    Deployment::new(Chain::Manta, "0xd71e5C1D217D12855B37fe60299273aaD91d6Cec"),
    Deployment::new(Chain::ZkSync, "0xeBd1e414eBb98522cfd932104bA41FAc10A4eF35"),
    Deployment::new(Chain::World, "0xa06e1351E2fD2D45b5D35633ca7eCF328684a109"),
    Deployment::new(Chain::Stable, "0xAc4615ffeC9dbf5EFE28DB0f98F0011E6df0DaBd"),
    Deployment::new(Chain::Hyperliquid, "0xd71e5C1D217D12855B37fe60299273aaD91d6Cec"),
    Deployment::new(Chain::Abstract, "0x634E831cE6D460c2CD5067Af98D6452Eb280E374"),
    Deployment::new(Chain::Mantle, "0xf366DA269047a06a7275a933c6D653409Bd6dE5e"),
    Deployment::new(Chain::Base, "0xa5F565650890fBA1824Ee0F21EbBbF660a179934"),
    Deployment::new(Chain::Plasma, "0xf17902d51FdfF7Bf50AacC78d6bB399BaF88b479"),
    Deployment::new(Chain::Arbitrum, "0xa5F565650890fBA1824Ee0F21EbBbF660a179934"),
    Deployment::new(Chain::Celo, "0x7F4babd2C7D35221e72Ab67Ea72Cba99573A0089"),
    Deployment::new(Chain::AvalancheC, "0xa5F565650890fBA1824Ee0F21EbBbF660a179934"),
    Deployment::new(Chain::Ink, "0x7F4babd2C7D35221e72Ab67Ea72Cba99573A0089"),
    Deployment::new(Chain::Linea, "0x00000000AA467EbA42a3d604B3d74d63B2b6C6Cb"),
    Deployment::new(Chain::Berachain, "0x7F4babd2C7D35221e72Ab67Ea72Cba99573A0089"),
];

pub(super) fn contracts(chain: Chain) -> Contracts {
    let (spenders, routers): (&[&'static str], &[&'static str]) = match chain {
        Chain::Mantle | Chain::Linea => (
            &[MANTLE_LINEA_DEPOSITORY, MANTLE_LINEA_APPROVAL_PROXY, MANTLE_LINEA_APPROVAL_PROXY_V3_2],
            &[MANTLE_LINEA_ERC20_ROUTER, MANTLE_LINEA_ERC20_ROUTER_V3_2],
        ),
        Chain::Tron => (&[TRON_DEPOSITORY], &[]),
        Chain::Ton => (&[TON_DEPOSITORY], &[]),
        Chain::Solana => (&[SOLANA_RELAY_DEPOSITORY_PROGRAM_ID, SOLANA_RELAY_FORWARDER_PROGRAM_ID], &[SOLANA_JUPITER_PROGRAM_ID]),
        _ if chain.chain_type() == ChainType::Ethereum => (&[DEPOSITORY, APPROVAL_PROXY, APPROVAL_PROXY_V3_2], &[ERC20_ROUTER, ERC20_ROUTER_V3_2]),
        _ => (&[], &[]),
    };
    Contracts {
        targets: [spenders, routers].concat().into_iter().chain(Deployment::contracts(RECEIVERS, chain).targets).collect(),
        spenders: spenders.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contracts() {
        let ethereum = contracts(Chain::Ethereum);
        assert!(ethereum.targets.contains(&DEPOSITORY) && ethereum.targets.contains(&APPROVAL_PROXY) && ethereum.targets.contains(&"0xa5F565650890fBA1824Ee0F21EbBbF660a179934"));
        let linea = contracts(Chain::Linea);
        assert!(linea.targets.contains(&MANTLE_LINEA_DEPOSITORY) && !linea.targets.contains(&DEPOSITORY));
        assert_eq!(contracts(Chain::Tron), Contracts::new(vec![TRON_DEPOSITORY]));
        assert_eq!(contracts(Chain::Ton), Contracts::new(vec![TON_DEPOSITORY]));
        assert_eq!(contracts(Chain::Bitcoin), Contracts::default(), "a Bitcoin deposit address is made per swap");
        assert!(ethereum.targets.contains(&ERC20_ROUTER) && !ethereum.spenders.contains(&ERC20_ROUTER), "the router is called, never approved");
        assert_eq!(
            contracts(Chain::Solana),
            Contracts {
                targets: vec![SOLANA_RELAY_DEPOSITORY_PROGRAM_ID, SOLANA_RELAY_FORWARDER_PROGRAM_ID, SOLANA_JUPITER_PROGRAM_ID],
                spenders: vec![SOLANA_RELAY_DEPOSITORY_PROGRAM_ID, SOLANA_RELAY_FORWARDER_PROGRAM_ID],
            },
            "token routes swap through Jupiter before the deposit"
        );
    }
}
