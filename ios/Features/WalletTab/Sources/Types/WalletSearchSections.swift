// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemNftEntry
import Primitives
import PrimitivesComponents
import Store

struct WalletSearchSections: Equatable {
    let pinnedAssets: [AssetData]
    let assets: [AssetData]

    let pinnedPerpetuals: [PerpetualData]
    let perpetuals: [PerpetualData]

    let nfts: [GemNftEntry]

    let lists: [AssetList]

    static func from(_ result: WalletSearchResult, nfts: [GemNftEntry]) -> WalletSearchSections {
        let assets = AssetsSections.from(result.assets)
        let (pinnedPerpetuals, perpetuals) = result.perpetuals.reduce(into: ([PerpetualData](), [PerpetualData]())) {
            if $1.metadata.isPinned {
                $0.0.append($1)
            } else {
                $0.1.append($1)
            }
        }
        return WalletSearchSections(
            pinnedAssets: assets.pinned,
            assets: assets.assets,
            pinnedPerpetuals: pinnedPerpetuals,
            perpetuals: perpetuals,
            nfts: nfts,
            lists: result.lists,
        )
    }
}
