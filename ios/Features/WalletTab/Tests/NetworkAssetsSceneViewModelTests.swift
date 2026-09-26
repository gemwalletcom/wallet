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
    func pinnedAndUnpinnedSplitOnTheirMetadata() {
        let model = NetworkAssetsSceneViewModel.mock()
        model.activeQuery.value = [
            .mock(asset: .mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20), metadata: .mock(isPinned: true)),
            .mock(asset: .mock(id: .mock(chain: .tron, tokenId: "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"), name: "Tether USD", symbol: "USDT", decimals: 6, type: .trc20), metadata: .mock(isPinned: false)),
            .mock(asset: .mock(id: .mock(chain: .solana, tokenId: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), name: "USD Coin", symbol: "USDC", decimals: 6, type: .spl), metadata: .mock(isPinned: false)),
        ]

        #expect(model.groups.pinned.count == 1)
        #expect(model.groups.unpinned.count == 2)
        #expect(model.groups.sections.showsPinned)
        #expect(model.groups.sections.showsUnpinned)
    }

    @Test
    func hiddenAssetsComeFromTheirOwnQuery() {
        let model = NetworkAssetsSceneViewModel.mock()
        model.hiddenQuery.value = [.mock(asset: .mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20), metadata: .mock(isPinned: false))]

        #expect(model.groups.sections.showsHidden)
        #expect(model.groups.sections.showsEmpty == false)
        #expect(model.assetIds.count == 1)
    }

    @Test
    func updatingBalancesAsksForEveryListedAsset() async {
        let service = GemWalletHomeServiceMock()
        let model = NetworkAssetsSceneViewModel.mock(service: service)
        let active = AssetData.mock(asset: .mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20), metadata: .mock(isPinned: false))
        let hidden = AssetData.mock(asset: .mock(id: .mock(chain: .ethereum, tokenId: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"), name: "USD Coin", symbol: "USDC", decimals: 6, type: .erc20), metadata: .mock(isPinned: false))
        model.activeQuery.value = [active]
        model.hiddenQuery.value = [hidden]

        await model.updateBalances()

        #expect(service.updatedBalances == [[active.asset.id.identifier, hidden.asset.id.identifier]])
    }

    @Test
    func pinningAndEnablingGoStraightToCore() async throws {
        let service = GemWalletHomeServiceMock()
        let model = NetworkAssetsSceneViewModel.mock(service: service)
        let assetId = AssetId.mock(chain: .ethereum)

        let toast = try await model.setAssetPinned(.mock(id: assetId, name: "Ethereum"), pinned: true)
        try await model.setAssetsEnabled([assetId], enabled: false)

        #expect(service.pinned.map(\.pinned) == [true])
        #expect(toast.text == .pinned(name: "Ethereum", pinned: true))
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
