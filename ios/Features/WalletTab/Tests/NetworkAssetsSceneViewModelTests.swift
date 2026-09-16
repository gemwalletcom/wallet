// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing
@testable import WalletTab
import WalletTabTestKit

@MainActor
struct NetworkAssetsSceneViewModelTests {
    @Test
    func anEmptyNetworkShowsOnlyTheEmptyState() {
        let model = NetworkAssetsSceneViewModel.mock()

        #expect(model.showEmpty)
        #expect(model.showPinned == false)
        #expect(model.showUnpinned == false)
        #expect(model.showHidden == false)
    }

    @Test
    func theNativeAssetIsNeverListed() {
        let model = NetworkAssetsSceneViewModel.mock()
        model.activeQuery.value = [.mock(asset: .mockEthereum()), .mock(asset: .mockEthereumUSDT(), metadata: .mock(isPinned: false))]

        #expect(model.active.count == 1)
        #expect(model.showUnpinned)
        #expect(model.showEmpty == false)
    }

    @Test
    func pinnedAndUnpinnedSplitOnTheirMetadata() {
        let model = NetworkAssetsSceneViewModel.mock()
        model.activeQuery.value = [
            .mock(asset: .mockEthereumUSDT(), metadata: .mock(isPinned: true)),
            .mock(asset: .mockTronUSDT(), metadata: .mock(isPinned: false)),
            .mock(asset: .mockSolanaUSDC(), metadata: .mock(isPinned: false)),
        ]

        #expect(model.pinned.count == 1)
        #expect(model.unpinned.count == 2)
        #expect(model.showPinned)
        #expect(model.showUnpinned)
    }

    @Test
    func hiddenAssetsComeFromTheirOwnQuery() {
        let model = NetworkAssetsSceneViewModel.mock()
        model.hiddenQuery.value = [.mock(asset: .mockEthereumUSDT(), metadata: .mock(isPinned: false))]

        #expect(model.showHidden)
        #expect(model.showEmpty == false)
        #expect(model.assetIds.count == 1)
    }

    @Test
    func updatingBalancesAsksForEveryListedAsset() async {
        let model = NetworkAssetsSceneViewModel.mock()
        let token = AssetData.mock(asset: .mockEthereumUSDT(), metadata: .mock(isPinned: false))
        model.activeQuery.value = [token]
        model.hiddenQuery.value = [token]

        await model.updateBalances()

        #expect(model.assetIds.count == 2)
    }

    @Test
    func pinningAndEnablingGoStraightToCore() async throws {
        let service = GemWalletHomeServiceMock()
        let model = NetworkAssetsSceneViewModel.mock(service: service)
        let assetId = AssetId.mock(.ethereum)

        try await model.setAssetPinned(assetId, pinned: true)
        try await model.setAssetsEnabled([assetId], enabled: false)

        #expect(service.pinned.map(\.pinned) == [true])
        #expect(service.enabled.map(\.enabled) == [false])
    }

    @Test
    func copyingAnAddressShowsTheToast() {
        let model = NetworkAssetsSceneViewModel.mock()

        model.onCopyAddress("copied")

        #expect(model.isPresentingToastMessage != nil)
    }

    @Test
    func managingAssetsCallsBack() {
        var calls = 0
        let model = NetworkAssetsSceneViewModel.mock(onManageAssets: { calls += 1 })

        model.onSelectManageAssets()

        #expect(calls == 1)
    }
}
