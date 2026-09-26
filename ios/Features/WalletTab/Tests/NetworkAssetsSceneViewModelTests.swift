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

        #expect(model.groups.sections.showsEmpty)
        #expect(model.groups.sections.showsPinned == false)
        #expect(model.groups.sections.showsUnpinned == false)
        #expect(model.groups.sections.showsHidden == false)
    }

    @Test
    func theNativeAssetIsNeverListed() {
        let model = NetworkAssetsSceneViewModel.mock()
        model.activeQuery.value = [
            .mock(asset: .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18)),
            .mock(asset: .mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20), metadata: .mock(isPinned: false)),
        ]

        #expect(model.groups.unpinned.map(\.asset.id) == [Asset.mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20).id])
        #expect(model.groups.sections.showsUnpinned)
        #expect(model.groups.sections.showsEmpty == false)
    }

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
        let model = NetworkAssetsSceneViewModel.mock()
        let token = AssetData.mock(asset: .mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20), metadata: .mock(isPinned: false))
        model.activeQuery.value = [token]
        model.hiddenQuery.value = [token]

        await model.updateBalances()

        #expect(model.assetIds.count == 2)
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
