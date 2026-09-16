// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemAssetConfigService
import GemstonePrimitives
import Primitives

public struct AssetsSections: Hashable, Sendable {
    public let pinned: [AssetData]
    public let assets: [AssetData]
    public let popular: [AssetData]
}

public extension AssetsSections {
    static func from(_ assets: [AssetData], showsPopular: Bool = false) -> AssetsSections {
        let sections = GemAssetConfigService.shared.assetSections(
            ids: assets.map(\.asset.id.identifier),
            pinnedIds: assets.filter(\.metadata.isPinned).map(\.asset.id.identifier),
            showsPopular: showsPopular,
        )
        let byId = Dictionary(assets.map { ($0.asset.id.identifier, $0) }, uniquingKeysWith: { first, _ in first })
        return AssetsSections(
            pinned: sections.pinned.compactMap { byId[$0] },
            assets: sections.assets.compactMap { byId[$0] },
            popular: sections.popular.compactMap { byId[$0] },
        )
    }
}
