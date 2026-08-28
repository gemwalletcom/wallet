// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServicesTestKit
import NFT
import PreferencesTestKit
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
    func recentActivityTypes() {
        #expect(WalletSearchSceneViewModel.mock(preferences: .mock(isPerpetualEnabled: true)).recentModel.query.request.types == RecentActivityType.allCases)
        #expect(WalletSearchSceneViewModel.mock(preferences: .mock(isPerpetualEnabled: false)).recentModel.query.request.types == RecentActivityType.allCases)
    }

    @Test
    func searchRequestInitialization() {
        #expect(WalletSearchSceneViewModel.mock(preferences: .mock(isPerpetualEnabled: true)).searchQuery.request.limit == 13)
        #expect(WalletSearchSceneViewModel.mock(preferences: .mock(isPerpetualEnabled: false)).searchQuery.request.limit == 13)
        #expect(WalletSearchSceneViewModel.mock(preferences: .mock(isPerpetualEnabled: true)).searchQuery.request.types == [.asset, .perpetual, .list, .nft])
        #expect(WalletSearchSceneViewModel.mock(preferences: .mock(isPerpetualEnabled: false)).searchQuery.request.types == [.asset, .perpetual, .list, .nft])
    }

    @Test
    func hasMoreAssets() {
        let model = WalletSearchSceneViewModel.mock(preferences: .mock(isPerpetualEnabled: true))

        model.searchQuery.value = .mock(assets: (0 ..< 12).map { _ in .mock() })
        #expect(model.hasMoreAssets == false)

        model.searchQuery.value = .mock(assets: (0 ..< 13).map { _ in .mock() })
        #expect(model.hasMoreAssets == true)
    }

    @Test
    func hasMorePerpetuals() {
        let model = WalletSearchSceneViewModel.mock(preferences: .mock(isPerpetualEnabled: true))

        model.searchQuery.value = .mock(perpetuals: (0 ..< 3).map { _ in .mock() })
        #expect(model.hasMorePerpetuals == false)

        model.searchQuery.value = .mock(perpetuals: (0 ..< 4).map { _ in .mock() })
        #expect(model.hasMorePerpetuals == true)
    }

    @Test(arguments: [
        Wallet.mock(type: .single, accounts: [.mock(chain: .bitcoin, address: "bc1")]),
        Wallet.mock(type: .view, accounts: [.mock(chain: .ethereum, address: "0x1")]),
        Wallet.mock(type: .multicoin, accounts: [.mock(chain: .ethereum, address: "0x1")]),
    ])
    func hidesPerpetualsForUnsupportedWallet(wallet: Wallet) {
        let model = WalletSearchSceneViewModel.mock(
            wallet: wallet,
            preferences: .mock(isPerpetualEnabled: true),
        )
        model.searchQuery.value = .mock(
            perpetuals: [
                .mock(metadata: .mock(isPinned: false)),
                .mock(metadata: .mock(isPinned: true)),
            ],
        )
        #expect(model.showPerpetuals == false)
        #expect(model.showPinnedPerpetuals == false)
    }

    @Test
    func listsSection() {
        let model = WalletSearchSceneViewModel.mock(preferences: .mock(isPerpetualEnabled: true))

        #expect(model.showLists == false)

        let list = AssetList(id: "stocks", name: "Stocks", count: 2)
        model.searchQuery.value = .mock(lists: [list])

        #expect(model.showLists == true)
        #expect(model.showEmpty == false)
        #expect(model.listDestination(for: list) == Scenes.AssetsResults(searchQuery: "", scope: .list("stocks"), title: "Stocks"))
    }

    @Test
    func hasMoreNFTs() {
        let model = WalletSearchSceneViewModel.mock(preferences: .mock(isPerpetualEnabled: true))

        model.searchQuery.value = .mock(nfts: (0 ..< 3).map { _ in .asset(.mock()) })
        #expect(model.hasMoreNFTs == false)

        model.searchQuery.value = .mock(nfts: (0 ..< 4).map { _ in .asset(.mock()) })
        #expect(model.hasMoreNFTs == true)
    }

    @Test
    func nftsSection() {
        let model = WalletSearchSceneViewModel.mock(preferences: .mock(isPerpetualEnabled: true))

        #expect(model.showNFTs == false)

        model.searchQuery.value = .mock(nfts: [
            .collection(NFTData(collection: .mock(), assets: [.mock(), .mock()])),
            .asset(.mock()),
        ])

        #expect(model.showNFTs == true)
        #expect(model.showEmpty == false)
        #expect(model.collectionsContent.items.count == 2)
    }

    @Test
    func pinAssetPinsThroughBalanceService() async {
        let pinned: (assetId: String, pinned: Bool) = await withCheckedContinuation { continuation in
            let model = WalletSearchSceneViewModel.mock(
                balanceService: .mock(onSetAssetPinned: { _, assetId, pinned in continuation.resume(returning: (assetId, pinned)) }),
            )
            model.onPinAsset(.mock(), value: true)
        }

        #expect(pinned.assetId == AssetId.mock().identifier)
        #expect(pinned.pinned)
    }
}
