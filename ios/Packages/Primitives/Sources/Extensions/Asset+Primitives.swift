// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

extension Asset: Identifiable {}

public extension Asset {
    var chain: Chain {
        id.chain
    }

    var tokenId: String? {
        id.tokenId
    }

    var feeAssetId: AssetId {
        switch id.type {
        case .native:
            id
        case .token:
            id.chain.assetId
        }
    }

    func getTokenId() throws -> String {
        try id.getTokenId()
    }

    func getTokenIdAsInt() throws -> Int {
        guard let tokenId, let tokenId = UInt64(tokenId) else {
            throw AnyError("tokenId is null")
        }
        return Int(tokenId)
    }
}

public extension [Asset] {
    var ids: [String] {
        map(\.id.identifier)
    }

    var assetIds: [AssetId] {
        map(\.id)
    }
}

public extension [Chain] {
    var ids: [AssetId] {
        compactMap(\.assetId)
    }
}

public extension AssetFull {
    var basic: AssetBasic {
        AssetBasic(
            asset: asset,
            properties: properties,
            score: score,
            price: nil,
        )
    }
}

// MARK: - Assets

public extension Asset {
    static func hypercoreUSDC() -> Asset {
        Asset(
            id: AssetId(chain: .hyperCore, tokenId: "perpetual::USDC"),
            name: "USDC",
            symbol: "USDC",
            decimals: 6,
            type: .perpetual,
        )
    }

    static func hypercoreSpotUSDC() -> Asset {
        Asset(
            id: AssetId(chain: .hyperCore, tokenId: "USDC::0x6d1e7cde53ba9467b783cb7c530ce054::0"),
            name: "USDC",
            symbol: "USDC",
            decimals: 8,
            type: .token,
        )
    }
}
