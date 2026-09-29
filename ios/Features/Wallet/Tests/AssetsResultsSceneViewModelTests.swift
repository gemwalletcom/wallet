// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing
@testable import Wallet
import WalletTestKit

@MainActor
struct AssetsResultsSceneViewModelTests {
    @Test
    func theRequestTakesItsKeyAndLimitFromCore() {
        let model = AssetsResultsSceneViewModel.mock()

        #expect(model.searchQuery.request.searchKey == "usdc")
        #expect(model.searchQuery.request.limit == 100)
    }

    @Test
    func theViewReadsAsLoadingUntilTheSearchAnswers() async {
        let service = GemAssetSelectionServiceMock()
        let model = AssetsResultsSceneViewModel.mock(service: service)

        _ = model.view
        #expect(service.resultsInputs.last?.isLoading == true)

        await model.refresh()
        _ = model.view
        #expect(service.resultsInputs.last?.isLoading == false)
    }

    @Test
    func aFailedSearchStopsLoading() async {
        let service = GemAssetSelectionServiceMock(error: AnyError("offline"))
        let model = AssetsResultsSceneViewModel.mock(service: service)

        await model.refresh()
        _ = model.view

        #expect(service.resultsInputs.last?.isLoading == false)
    }

    @Test
    func theViewReceivesTheScopeAndEveryObservedRow() {
        let service = GemAssetSelectionServiceMock()
        let model = AssetsResultsSceneViewModel.mock(service: service, request: WalletSearchQuery(walletId: .mock(), scope: .list("trending"), types: [.asset, .perpetual]))
        let pinned = AssetData.mock(asset: .mock(id: .mock(chain: .ethereum)), metadata: .mock(isPinned: true))
        let other = AssetData.mock(asset: .mock(id: .mock(chain: .bitcoin)), metadata: .mock(isPinned: false))
        model.searchQuery.value = .mock(assets: [pinned, other], perpetuals: [.mock()])

        _ = model.view
        let input = service.resultsInputs.last

        #expect(input?.scope == .list(id: "trending"))
        #expect(input?.assetIds == [pinned.asset.id, other.asset.id])
        #expect(input?.pinnedAssetIds == [pinned.asset.id])
        #expect(input?.perpetuals.count == 1)
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
        let assetId = AssetId.mock(chain: .ethereum)

        _ = try await model.setAssetPinned(.mock(id: assetId), pinned: true)
        try await model.setAssetsEnabled([assetId], enabled: false)
        _ = try await model.setPerpetualPinned(.mock(), pinned: true)

        #expect(calls.pinned == [true])
        #expect(calls.enabled == [false])
        #expect(service.pinnedPerpetuals.map(\.pinned) == [true])
    }
}

private final class CallRecorder: @unchecked Sendable {
    private(set) var pinned: [Bool] = []
    private(set) var enabled: [Bool] = []

    func record(assetId _: AssetId, pinned value: Bool) {
        pinned.append(value)
    }

    func record(assetIds _: [AssetId], enabled value: Bool) {
        enabled.append(value)
    }
}
