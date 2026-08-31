// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public struct AssetsSections: Hashable, Sendable {
    public let pinned: [AssetData]
    public let assets: [AssetData]
    public let popular: [AssetData]
}

public extension AssetsSections {
    static func from(_ assets: [AssetData], popularIds: Set<AssetId> = []) -> AssetsSections {
        let popular = assets.filter { popularIds.contains($0.asset.id) }
        return AssetsSections(
            pinned: assets.filter(\.metadata.isPinned),
            assets: assets.filter { !$0.metadata.isPinned && !popularIds.contains($0.asset.id) },
            popular: popular,
        )
    }
}
