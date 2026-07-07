// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public struct AssetsSections: Hashable, Sendable {
    public let pinned: [AssetData]
    public let assets: [AssetData]
    public let popular: [AssetData]

    private static let popularChains = Set<AssetId>(arrayLiteral: Chain.bitcoin.assetId, Chain.ethereum.assetId, Chain.solana.assetId)
}

public extension AssetsSections {
    static func from(_ assets: [AssetData], enablePopular: Bool = false) -> AssetsSections {
        let popular = enablePopular ? assets.filter { Self.popularChains.contains($0.asset.id) } : []
        let popularIds = Set(popular.map(\.asset.id))
        return AssetsSections(
            pinned: assets.filter(\.metadata.isPinned),
            assets: assets.filter { !$0.metadata.isPinned && !popularIds.contains($0.asset.id) },
            popular: popular,
        )
    }
}
