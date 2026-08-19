// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension Asset {
    static func mock(
        id: AssetId = .mock(),
        name: String = "Bitcoin",
        symbol: String = "BTC",
        decimals: Int32 = 8,
        type: AssetType = .native,
    ) -> Asset {
        Asset(
            id: id,
            name: name,
            symbol: symbol,
            decimals: decimals,
            type: type,
        )
    }

    static func mockEthereum() -> Asset {
        .mock(
            id: AssetId(chain: .ethereum, tokenId: .none),
            name: "Ethereum",
            symbol: "ETH",
            decimals: 18,
            type: .native,
        )
    }

    static func mockBNB() -> Asset {
        .mock(
            id: AssetId(chain: .smartChain, tokenId: .none),
            name: "BNB",
            symbol: "BNB",
            decimals: 18,
            type: .native,
        )
    }

    static func mockXRP() -> Asset {
        .mock(
            id: AssetId(chain: .xrp, tokenId: .none),
            name: "XRP",
            symbol: "XRP",
            decimals: 8,
            type: .native,
        )
    }

    static func mockEthereumUSDT() -> Asset {
        .mock(
            id: AssetId(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"),
            name: "Tether",
            symbol: "USDT",
            decimals: 6,
            type: .erc20,
        )
    }

    static func mockTron() -> Asset {
        .mock(
            id: AssetId(chain: .tron, tokenId: .none),
            name: "TRON",
            symbol: "TRX",
            decimals: 6,
            type: .native,
        )
    }

    static func mockTronUSDT() -> Asset {
        Asset(
            id: AssetId(chain: .tron, tokenId: "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"),
            name: "Tether USD",
            symbol: "USDT",
            decimals: 6,
            type: .trc20,
        )
    }

    static func mockNear() -> Asset {
        Asset(
            id: AssetId(chain: .near, tokenId: nil),
            name: "NEAR",
            symbol: "NEAR",
            decimals: 24,
            type: .native,
        )
    }

    static func mockSUI() -> Asset {
        Asset(
            id: AssetId(chain: .sui, tokenId: nil),
            name: "Sui",
            symbol: "SUI",
            decimals: 9,
            type: .native,
        )
    }

    static func mockSolana() -> Asset {
        .mock(
            id: AssetId(chain: .solana, tokenId: .none),
            name: "Solana",
            symbol: "SOL",
            decimals: 9,
            type: .native,
        )
    }

    static func mockSolanaUSDC() -> Asset {
        .mock(
            id: AssetId(chain: .solana, tokenId: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
            name: "USD Coin",
            symbol: "USDC",
            decimals: 6,
            type: .spl,
        )
    }

    static func mockHypercore() -> Asset {
        Asset(
            id: AssetId(chain: .hyperCore, tokenId: nil),
            name: "Hyperliquid",
            symbol: "HYPE",
            decimals: 8,
            type: .native,
        )
    }

    static func mockHypercoreUSDC() -> Asset {
        .mock(
            id: AssetId(chain: .hyperCore, tokenId: "perpetual::USDC"),
            name: "USDC",
            symbol: "USDC",
            decimals: 6,
            type: .perpetual,
        )
    }

    static func mockHypercoreSpotUSDC() -> Asset {
        .mock(
            id: AssetId(chain: .hyperCore, tokenId: "USDC::0x6d1e7cde53ba9467b783cb7c530ce054::0"),
            name: "USDC",
            symbol: "USDC",
            decimals: 8,
            type: .token,
        )
    }

    static func mockTempoUSDC() -> Asset {
        .mock(
            id: AssetId(chain: .tempo, tokenId: "0x20C000000000000000000000b9537d11c60E8b50"),
            name: "Bridged USDC",
            symbol: "USDC.e",
            decimals: 6,
            type: .tip20,
        )
    }

    static func mockTempoPathUSD() -> Asset {
        .mock(
            id: AssetId(chain: .tempo, tokenId: "0x20C0000000000000000000000000000000000000"),
            name: "pathUSD",
            symbol: "pathUSD",
            decimals: 6,
            type: .tip20,
        )
    }
}
