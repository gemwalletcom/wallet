// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemAssetConfigService
import enum Gemstone.GemAssetSectionKind
import GemstonePrimitives
import Primitives

public struct AssetsSection: Hashable, Sendable {
    public let kind: GemAssetSectionKind
    public let assets: [AssetData]
}

public struct AssetsSections: Hashable, Sendable {
    public let sections: [AssetsSection]

    public var pinned: [AssetData] { values(.pinned) }
    public var assets: [AssetData] { values(.assets) }
    public var popular: [AssetData] { values(.popular) }

    private func values(_ kind: GemAssetSectionKind) -> [AssetData] {
        sections.first { $0.kind == kind }?.assets ?? []
    }
}

public extension AssetsSections {
    static func from(_ assets: [AssetData], showsPopular: Bool = false) -> AssetsSections {
        let sections = GemAssetConfigService.shared.assetSections(
            ids: assets.map(\.asset.id.identifier),
            pinnedIds: assets.filter(\.metadata.isPinned).map(\.asset.id.identifier),
            showsPopular: showsPopular,
        )
        let byId = Dictionary(assets.map { ($0.asset.id.identifier, $0) }, uniquingKeysWith: { first, _ in first })
        return AssetsSections(sections: sections.map { AssetsSection(kind: $0.kind, assets: $0.assetIds.compactMap { byId[$0] }) })
    }
}
