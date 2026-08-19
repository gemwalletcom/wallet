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
            price: price,
        )
    }
}
