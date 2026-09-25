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
    func searchRequestInitialization() {
        #expect(WalletSearchSceneViewModel.mock().searchQuery.request.limit == 13)
        #expect(WalletSearchSceneViewModel.mock().searchQuery.request.types == [.asset, .perpetual, .list, .nft])
    }

    @Test
    func hasMoreAssets() {
        let model = WalletSearchSceneViewModel.mock()
        let unpinned = { AssetData.mock(metadata: .mock(isPinned: false)) }

        model.searchQuery.value = .mock(assets: (0 ..< 12).map { _ in unpinned() })
        #expect(model.derived.view.hasMoreAssets == false)

        model.searchQuery.value = .mock(assets: (0 ..< 13).map { _ in unpinned() })
        #expect(model.derived.view.hasMoreAssets == true)
    }

    @Test
    func hasMoreAssetsCountsOnlyTheAssetsThePreviewShows() {
        let model = WalletSearchSceneViewModel.mock()
        model.searchQuery.value = .mock(assets: (0 ..< 13).map { _ in AssetData.mock(metadata: .mock(isPinned: true)) })

        #expect(model.derived.previewAssets.isEmpty)
        #expect(model.derived.view.hasMoreAssets == false)
    }

    @Test
    func hasMorePerpetuals() {
        let model = WalletSearchSceneViewModel.mock()

        model.searchQuery.value = .mock(perpetuals: (0 ..< 3).map { _ in .mock() })
        #expect(model.derived.view.hasMorePerpetuals == false)

        model.searchQuery.value = .mock(perpetuals: (0 ..< 4).map { _ in .mock() })
        #expect(model.derived.view.hasMorePerpetuals == true)
    }

    @Test
    func hidesPerpetualsWhenTheServiceSaysSo() {
        let service = GemAssetSelectionServiceMock()
        service.perpetualsShown = false
        let model = WalletSearchSceneViewModel.mock(service: service)
        model.searchQuery.value = .mock(
            perpetuals: [
                .mock(metadata: .mock(isPinned: false)),
                .mock(metadata: .mock(isPinned: true)),
            ],
        )
        #expect(model.derived.view.state.showsPerpetuals == false)
        #expect(model.derived.view.state.showsPinnedPerpetuals == false)
    }

    @Test
    func listsSection() {
        let model = WalletSearchSceneViewModel.mock()

        #expect(model.derived.view.state.showsLists == false)

        let list = AssetList(id: "stocks", name: "Stocks", count: 2)
        model.searchQuery.value = .mock(lists: [list])

        #expect(model.derived.view.state.showsLists == true)
        #expect(model.listDestination(for: list) == Scenes.AssetsResults(searchQuery: "", scope: .list("stocks"), title: "Stocks"))
    }

    @Test
    func hasMoreNFTs() {
        let service = GemAssetSelectionServiceMock()
        let model = WalletSearchSceneViewModel.mock(service: service)

        service.nftSearchItems = (0 ..< 3).map { _ in .mock(item: .asset(data: NFTAssetData.mock().toGem())) }
        #expect(model.derived.view.hasMoreNfts == false)

        service.nftSearchItems = (0 ..< 4).map { _ in .mock(item: .asset(data: NFTAssetData.mock().toGem())) }
        #expect(model.derived.view.hasMoreNfts == true)
    }

    @Test
    func nftsSection() {
        let service = GemAssetSelectionServiceMock()
        let model = WalletSearchSceneViewModel.mock(service: service)

        #expect(model.derived.view.state.showsNfts == false)

        service.nftSearchItems = [
            .mock(item: .collection(data: NFTData.mock(assets: [.mock(), .mock()]).toGem())),
            .mock(item: .asset(data: NFTAssetData.mock().toGem())),
        ]

        #expect(model.derived.view.state.showsNfts == true)
        #expect(model.derived.collectionsContent.items.count == 2)
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
    func addingATokenNeedsBothTokenSupportAndAChainToAddItTo() {
        let service = GemAssetSelectionServiceMock()
        let model = WalletSearchSceneViewModel.mock(service: service)

        #expect(model.derived.view.showsAddToken == false, "there is no chain to add a token to")

        service.filterChainsResult = [Primitives.Chain.ethereum.toGem()]
        #expect(model.derived.view.showsAddToken)

        service.tokensSupported = false
        #expect(model.derived.view.showsAddToken == false)
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
