// Copyright (c). Gem Wallet. All rights reserved.

import Components
import PrimitivesComponents
import GemstonePrimitives
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing
@testable import WalletTab
import WalletTabTestKit

@MainActor
struct AssetsResultsSceneViewModelTests {
    @Test
    func theRequestTakesItsKeyAndLimitFromCore() {
        let model = AssetsResultsSceneViewModel.mock()

        #expect(model.searchQuery.request.searchKey == "usdc")
        #expect(model.searchQuery.request.limit == 100)
    }

    @Test
    func aListScopeSearchesByItsTag() {
        let model = AssetsResultsSceneViewModel.mock(request: WalletSearchRequest(walletId: .mock(), scope: .list("trending"), types: [.asset]))

        #expect(model.searchQuery.request.searchKey == "tag:trending")
    }

    @Test
    func noResultsReadAsEmptyOnceTheSearchFinished() async {
        let model = AssetsResultsSceneViewModel.mock()

        await model.refresh()

        #expect(model.showEmpty)
        if case .empty = model.searchState {} else { Issue.record("expected the empty state, got \(model.searchState)") }
    }

    @Test
    func aFailedSearchStillLeavesTheEmptyState() async {
        let model = AssetsResultsSceneViewModel.mock(service: GemAssetSelectionServiceMock(error: AnyError("offline")))

        await model.refresh()

        #expect(model.showEmpty)
    }

    @Test
    func assetsAndPinnedAssetsSplitOnTheirMetadata() {
        let model = AssetsResultsSceneViewModel.mock()
        model.searchQuery.value = .mock(assets: [.mock(metadata: .mock(isPinned: true)), .mock(metadata: .mock(isPinned: false))])

        #expect(model.showPinned)
        #expect(model.showAssets)
        #expect(model.showEmpty == false)
    }

    @Test
    func perpetualsAreOfferedOnlyInAListScopeAndOnlyWhenCoreAllowsThem() {
        let service = GemAssetSelectionServiceMock()
        let listModel = AssetsResultsSceneViewModel.mock(service: service, request: WalletSearchRequest(walletId: .mock(), scope: .list("trending"), types: [.perpetual]))
        listModel.searchQuery.value = .mock(perpetuals: [PerpetualData.mock()])

        #expect(listModel.showPerpetuals)

        service.perpetualsShown = false
        #expect(listModel.showPerpetuals == false)

        let allModel = AssetsResultsSceneViewModel.mock()
        allModel.searchQuery.value = .mock(perpetuals: [PerpetualData.mock()])
        #expect(allModel.showPerpetuals == false)
    }

    @Test
    func selectingAnAssetCallsBack() {
        var selected: [Asset] = []
        let model = AssetsResultsSceneViewModel.mock(onSelectAsset: { selected.append($0) })

        model.onSelectAsset(.mock())

        #expect(selected.count == 1)
    }

    @Test
    func pinningAndEnablingGoStraightToCore() async throws {
        let calls = CallRecorder()
        let service = GemAssetSelectionServiceMock(
            onSetAssetsEnabled: { ids, value in calls.record(assetIds: ids, enabled: value) },
            onSetAssetPinned: { id, value in calls.record(assetId: id, pinned: value) },
        )
        let model = AssetsResultsSceneViewModel.mock(service: service)
        let assetId = AssetId.mock(.ethereum)

        try await model.setAssetPinned(assetId, pinned: true)
        try await model.setAssetsEnabled([assetId], enabled: false)
        try await model.setPerpetualPinned(PerpetualId(provider: .hypercore, symbol: "BTC"), pinned: true)

        #expect(calls.pinned == [true])
        #expect(calls.enabled == [false])
        #expect(service.pinnedPerpetuals.map { $0.pinned } == [true])
    }
}

private final class CallRecorder: @unchecked Sendable {
    private(set) var pinned: [Bool] = []
    private(set) var enabled: [Bool] = []

    func record(assetId _: String, pinned value: Bool) {
        pinned.append(value)
    }

    func record(assetIds _: [String], enabled value: Bool) {
        enabled.append(value)
    }
}
