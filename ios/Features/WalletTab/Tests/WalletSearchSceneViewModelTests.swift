// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import NFT
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing
@testable import WalletTab
import WalletTabTestKit

@MainActor
struct WalletSearchSceneViewModelTests {
    @Test
    func hasMoreAssetsCountsOnlyTheAssetsThePreviewShows() {
        let model = WalletSearchSceneViewModel.mock()
        model.searchQuery.value = .mock(assets: (0 ..< 13).map { _ in AssetData.mock(metadata: .mock(isPinned: true)) })

        #expect(model.derived.previewAssets.isEmpty)
        #expect(model.derived.view.hasMoreAssets == false)
    }

    @Test
    func listsSection() {
        let model = WalletSearchSceneViewModel.mock()

        #expect(model.derived.view.state.showsLists == false)

        let list = AssetList(id: "stocks", name: "Stocks", count: 2)
        model.searchQuery.value = .mock(lists: [list])

        let row = model.derived.sections.lists[0]
        #expect(model.derived.view.state.showsLists == true)
        #expect(row.listItem.subtitle == "2")
        #expect(model.listDestination(for: row) == Scenes.AssetsResults(searchQuery: "", scope: .list("stocks"), title: "Stocks"))
    }

    @Test
    func nftsSection() {
        let service = GemAssetSelectionServiceMock()
        let model = WalletSearchSceneViewModel.mock(service: service)

        #expect(model.derived.view.state.showsNfts == false)

        service.nftSearchItems = [
            .mock(item: .collection(data: NFTData.mock(assets: [.mock(), .mock()]).toGem()), row: .mock(id: "collection", isVerified: true)),
            .mock(item: .asset(data: NFTAssetData.mock().toGem()), row: .mock(id: "asset", isVerified: true)),
        ]

        #expect(model.derived.view.state.showsNfts == true)
        #expect(model.derived.previewNFTs.count == 2)
    }

    @Test
    func aMatchingListAloneIsNotAnEmptySearch() {
        let model = WalletSearchSceneViewModel.mock()
        model.searchQuery.value = .mock(lists: [AssetList(id: "stocks", name: "Stocks", count: 2)])

        if case .results = model.searchState(model.derived) {} else {
            Issue.record("expected results, got \(model.searchState(model.derived))")
        }
    }

    @Test
    func pinAssetPinsThroughTheService() async {
        let pinned: (assetId: String, pinned: Bool) = await withCheckedContinuation { continuation in
            let model = WalletSearchSceneViewModel.mock(
                service: GemAssetSelectionServiceMock(onSetAssetPinned: { assetId, pinned in continuation.resume(returning: (assetId, pinned)) }),
            )
            model.onPinAsset(.mock(), value: true)
        }

        #expect(pinned.assetId == AssetId.mock().identifier)
        #expect(pinned.pinned)
    }
}
