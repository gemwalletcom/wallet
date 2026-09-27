// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemNftEntry
import struct Gemstone.GemPerpetualMarketItem
import struct Gemstone.GemSearchListRow
import func Gemstone.perpetualMarketSections
import func Gemstone.searchListRows
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Store

struct WalletSearchSections: Equatable {
    let pinnedAssets: [AssetData]
    let assets: [AssetData]

    let pinnedPerpetuals: [GemPerpetualMarketItem]
    let perpetuals: [GemPerpetualMarketItem]

    let nfts: [GemNftEntry]

    let lists: [GemSearchListRow]

    static func from(_ result: WalletSearchResult, nfts: [GemNftEntry]) -> WalletSearchSections {
        let assets = AssetsSections.from(result.assets)
        let perpetuals = perpetualMarketSections(markets: result.perpetuals.map { $0.toGem() })
        return WalletSearchSections(
            pinnedAssets: assets.pinned,
            assets: assets.assets,
            pinnedPerpetuals: perpetuals.pinned,
            perpetuals: perpetuals.markets,
            nfts: nfts,
            lists: searchListRows(lists: result.lists.map { $0.toGem() }),
        )
    }
}
