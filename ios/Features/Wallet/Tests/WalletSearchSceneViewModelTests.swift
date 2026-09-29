// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemSearchListRow
import GemstonePrimitives
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing
@testable import Wallet
import WalletTestKit

@MainActor
struct WalletSearchSceneViewModelTests {
    @Test
    func theViewReceivesEveryObservedRowWithItsPinnedFlag() {
        let service = GemAssetSelectionServiceMock()
        let model = WalletSearchSceneViewModel.mock(service: service)
        let pinned = AssetData.mock(asset: .mock(id: .mock(chain: .ethereum)), metadata: .mock(isPinned: true))
        let other = AssetData.mock(asset: .mock(id: .mock(chain: .bitcoin)), metadata: .mock(isPinned: false))
        model.searchQuery.value = .mock(
            assets: [pinned, other],
            perpetuals: [.mock()],
            lists: [AssetList(id: "stocks", name: "Stocks", count: 2)],
        )

        _ = model.view
        let input = service.searchInputs.last

        #expect(input?.assetIds == [pinned.asset.id, other.asset.id])
        #expect(input?.pinnedAssetIds == [pinned.asset.id])
        #expect(input?.perpetuals.count == 1)
        #expect(input?.lists.map(\.id) == ["stocks"])
    }

    @Test
    func aTypedQueryReadsAsLoadingUntilTheSearchAnswers() async {
        let service = GemAssetSelectionServiceMock()
        let model = WalletSearchSceneViewModel.mock(service: service)

        model.searchableQuery = "btc"
        model.onChangeSearchQuery("", "btc")
        _ = model.view
        #expect(service.searchInputs.last?.query == "btc")
        #expect(service.searchInputs.last?.isLoading == true)

        await model.onSearch(query: "btc")
        _ = model.view
        #expect(service.searchInputs.last?.isLoading == false)
    }

    @Test
    func assetIdsResolveToTheirRowsInCoreOrder() {
        let model = WalletSearchSceneViewModel.mock()
        let ethereum = AssetData.mock(asset: .mock(id: .mock(chain: .ethereum)))
        let bitcoin = AssetData.mock(asset: .mock(id: .mock(chain: .bitcoin)))
        model.searchQuery.value = .mock(assets: [ethereum, bitcoin])

        #expect(model.assets([bitcoin.asset.id, ethereum.asset.id]).map(\.asset.id) == [bitcoin.asset.id, ethereum.asset.id])
    }

    @Test
    func aListRowOpensItsResults() {
        let model = WalletSearchSceneViewModel.mock()
        let row = GemSearchListRow(list: AssetList(id: "stocks", name: "Stocks", count: 2).toGem(), subtitle: "2", imageUrl: "")

        #expect(model.listDestination(for: row) == Scenes.AssetsResults(searchQuery: "", scope: .list("stocks"), title: "Stocks"))
    }

    @Test
    func pinAssetPinsThroughTheService() async {
        let pinned: (assetId: AssetId, pinned: Bool) = await withCheckedContinuation { continuation in
            let model = WalletSearchSceneViewModel.mock(
                service: GemAssetSelectionServiceMock(onSetAssetPinned: { assetId, pinned in continuation.resume(returning: (assetId, pinned)) }),
            )
            model.onPinAsset(.mock(), value: true)
        }

        #expect(pinned.assetId == AssetId.mock())
        #expect(pinned.pinned)
    }
}
