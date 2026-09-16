// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing
@testable import WalletTab

@MainActor
struct NetworkAssetsSceneViewModelTests {
    private func token(_ contract: String = "0xtoken", pinned: Bool = false) -> AssetData {
        AssetData.mock(
            asset: .mock(id: AssetId(chain: .ethereum, tokenId: contract), type: .erc20),
            metadata: .mock(isPinned: pinned),
        )
    }

    private func model(
        service: GemWalletHomeServiceMock = GemWalletHomeServiceMock(),
        onManageAssets: @escaping () -> Void = {},
    ) -> NetworkAssetsSceneViewModel {
        NetworkAssetsSceneViewModel(wallet: .mock(), chain: .ethereum, service: service, onManageAssets: onManageAssets)
    }

    @Test
    func anEmptyNetworkShowsOnlyTheEmptyState() {
        let model = model()

        #expect(model.showEmpty)
        #expect(model.showPinned == false)
        #expect(model.showUnpinned == false)
        #expect(model.showHidden == false)
    }

    @Test
    func theNativeAssetIsNeverListed() {
        let model = model()
        model.activeQuery.value = [AssetData.mock(asset: .mock(id: .mock(.ethereum), type: .native)), token()]

        #expect(model.active.count == 1)
        #expect(model.showUnpinned)
        #expect(model.showEmpty == false)
    }

    @Test
    func pinnedAndUnpinnedSplitOnTheirMetadata() {
        let model = model()
        model.activeQuery.value = [token("0xa", pinned: true), token("0xb"), token("0xc")]

        #expect(model.pinned.count == 1)
        #expect(model.unpinned.count == 2)
        #expect(model.showPinned)
        #expect(model.showUnpinned)
    }

    @Test
    func hiddenAssetsComeFromTheirOwnQuery() {
        let model = model()
        model.hiddenQuery.value = [token()]

        #expect(model.showHidden)
        #expect(model.showEmpty == false)
        #expect(model.assetIds.count == 1)
    }

    @Test
    func updatingBalancesAsksForEveryListedAsset() async {
        let model = model()
        model.activeQuery.value = [token()]
        model.hiddenQuery.value = [token()]

        await model.updateBalances()

        #expect(model.assetIds.count == 2)
    }

    @Test
    func pinningAndEnablingGoStraightToCore() async throws {
        let service = GemWalletHomeServiceMock()
        let model = model(service: service)
        let assetId = AssetId.mock(.ethereum)

        try await model.setAssetPinned(assetId, pinned: true)
        try await model.setAssetsEnabled([assetId], enabled: false)

        #expect(service.pinned.map(\.pinned) == [true])
        #expect(service.enabled.map(\.enabled) == [false])
    }

    @Test
    func copyingAnAddressShowsTheToast() {
        let model = model()

        model.onCopyAddress("copied")

        #expect(model.isPresentingToastMessage != nil)
    }

    @Test
    func managingAssetsCallsBack() {
        var calls = 0
        let model = model(onManageAssets: { calls += 1 })

        model.onSelectManageAssets()

        #expect(calls == 1)
    }
}
