// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import Testing

struct SelectAssetTypeTests {
    @Test
    func recentActivityTypes() {
        #expect(SelectAssetType.swap(.pay).recentActivityTypes == [.swapSelect, .swap])
        #expect(SelectAssetType.swap(.receive(chains: [], assetIds: [])).recentActivityTypes == [.swapSelect, .swap])
        #expect(SelectAssetType.receive(.asset).recentActivityTypes == RecentActivityType.allCases)
        #expect(SelectAssetType.buy.recentActivityTypes == RecentActivityType.allCases)
    }

    @Test
    func recentActivityData() {
        let assetId = AssetId(chain: .bitcoin)
        #expect(SelectAssetType.swap(.pay).recentActivityData(assetId: assetId)?.type == .swapSelect)
        #expect(SelectAssetType.swap(.receive(chains: [], assetIds: [])).recentActivityData(assetId: assetId)?.type == .swapSelect)
        #expect(SelectAssetType.receive(.asset).recentActivityData(assetId: assetId)?.type == .receive)
        #expect(SelectAssetType.buy.recentActivityData(assetId: assetId)?.type == .fiatBuy)
        #expect(SelectAssetType.send.recentActivityData(assetId: assetId) == nil)
    }
}
