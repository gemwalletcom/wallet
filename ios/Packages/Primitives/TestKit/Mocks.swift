// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

public extension AssetId {
    static func mock(
        chain: Chain = .bitcoin,
        tokenId: String? = nil,
    ) -> AssetId {
        AssetId(chain: chain, tokenId: tokenId)
    }
}

public extension WalletId {
    static func mock(
        address: String = "0x0000000000000000000000000000000000000000",
    ) -> WalletId {
        .multicoin(address: address)
    }
}

public extension TransactionId {
    static func mock(
        chain: Chain = .bitcoin,
        hash: String = "tx-id",
    ) -> TransactionId {
        TransactionId(chain: chain, hash: hash)
    }
}

public extension NFTAssetId {
    static func mock(
        chain: Chain = .bitcoin,
        contractAddress: String = "0xcontract",
        tokenId: String = "1",
    ) -> NFTAssetId {
        NFTAssetId(chain: chain, contractAddress: contractAddress, tokenId: tokenId)
    }
}

public extension NFTCollectionId {
    static func mock(
        chain: Chain = .bitcoin,
        contractAddress: String = "0xcontract",
    ) -> NFTCollectionId {
        NFTCollectionId(chain: chain, contractAddress: contractAddress)
    }
}

public extension PerpetualId {
    static func mock(
        provider: PerpetualProvider = .hypercore,
        symbol: String = "BTC",
    ) -> PerpetualId {
        PerpetualId(provider: provider, symbol: symbol)
    }
}

public extension AssetAddress {
    static func mock(
        asset: Asset = .mock(),
        address: String = "",
    ) -> AssetAddress {
        AssetAddress(
            asset: asset,
            address: address,
        )
    }
}

public extension PriceData {
    static func mock(
        asset: Asset = .mock(),
        price: Price? = nil,
        priceAlerts: [PriceAlert] = [],
        market: AssetMarket? = nil,
        links: [AssetLink] = [],
    ) -> PriceData {
        PriceData(
            asset: asset,
            price: price,
            priceAlerts: priceAlerts,
            market: market,
            links: links,
        )
    }
}

public extension AutocloseOpenData {
    static func mock(
        assetId: AssetId = .mock(),
        symbol: String = "",
        direction: PerpetualDirection = .short,
        marketPrice: Double = 0,
        leverage: UInt8 = 0,
        size: Double = 0,
        assetDecimals: Int32 = 0,
        takeProfit: String? = nil,
        stopLoss: String? = nil,
    ) -> AutocloseOpenData {
        AutocloseOpenData(
            assetId: assetId,
            symbol: symbol,
            direction: direction,
            marketPrice: marketPrice,
            leverage: leverage,
            size: size,
            assetDecimals: assetDecimals,
            takeProfit: takeProfit,
            stopLoss: stopLoss,
        )
    }
}
