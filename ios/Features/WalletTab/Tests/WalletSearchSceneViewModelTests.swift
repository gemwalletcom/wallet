// Copyright (c). Gem Wallet. All rights reserved.

import BalanceServiceTestKit
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
        #expect(WalletSearchSceneViewModel.mock(preferences: .mock(isPerpetualEnabled: true)).recentsQuery.request.types == RecentActivityType.allCases)
        #expect(WalletSearchSceneViewModel.mock(preferences: .mock(isPerpetualEnabled: false)).recentsQuery.request.types == RecentActivityType.allCases)
    }

    @Test
    func searchRequestInitialization() {
        #expect(WalletSearchSceneViewModel.mock(preferences: .mock(isPerpetualEnabled: true)).searchQuery.request.limit == 13)
        #expect(WalletSearchSceneViewModel.mock(preferences: .mock(isPerpetualEnabled: false)).searchQuery.request.limit == 13)
        #expect(WalletSearchSceneViewModel.mock(preferences: .mock(isPerpetualEnabled: true)).searchQuery.request.types == [.asset, .perpetual, .list])
        #expect(WalletSearchSceneViewModel.mock(preferences: .mock(isPerpetualEnabled: false)).searchQuery.request.types == [.asset, .perpetual, .list])
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
    func pinAssetEnablesAsset() async {
        let db = DB.mockAssets()
        let enabledAssetIds: [AssetId] = await withCheckedContinuation { continuation in
            let model = WalletSearchSceneViewModel.mock(
                assetsEnabler: .mock(onEnableAssets: { _, assetIds, _ in continuation.resume(returning: assetIds) }),
                balanceService: .mock(balanceStore: .mock(db: db)),
            )
            model.onSelectPinAsset(.mock(metadata: .mock(isBalanceEnabled: false, isPinned: false)), value: true)
        }

        #expect(enabledAssetIds == [AssetId.mock()])
    }
}
