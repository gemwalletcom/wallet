// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

public extension AssetBasic {
    static func mock(
        asset: Asset = .mock(),
        properties: AssetProperties = .mock(),
        score: AssetScore = .mock(),
        price: Price? = nil,
    ) -> Self {
        AssetBasic(
            asset: asset,
            properties: properties,
            score: score,
            price: price,
        )
    }
}

public extension [AssetBasic] {
    static func mock() -> Self {
        [
            .mock(asset: .mock(name: "Bitcoin", symbol: "BTC", decimals: 8)),
            .mock(asset: .mock(id: .mock(chain: .smartChain), name: "BNB", symbol: "BNB", decimals: 18)),
            .mock(asset: .mock(id: .mock(chain: .tron), name: "TRON", symbol: "TRX", decimals: 6)),
            .mock(asset: .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18)),
            .mock(asset: .mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20)),
        ]
    }
}
