// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemPerpetualServiceProtocol
import Store
import StoreTestKit
import GemstoneServices
import GemstoneServicesTestKit
import protocol Gemstone.GemSearchServiceProtocol
import GemstonePrimitivesTestKit
import Preferences
import PreferencesTestKit
import Primitives
import PrimitivesTestKit
import WalletTab

public extension WalletSearchSceneViewModel {
    @MainActor
    static func mock(
        wallet: Wallet = .mock(),
        searchService: any GemSearchServiceProtocol = GemSearchServiceMock(),
        recentActivityStore: RecentActivityStore = .mock(),
        assetsEnabler: any AssetsEnabler = .mock(),
        perpetualService: any GemPerpetualServiceProtocol = GemPerpetualServiceMock(),
        preferences: ObservablePreferences = .mock(),
    ) -> WalletSearchSceneViewModel {
        WalletSearchSceneViewModel(
            wallet: wallet,
            searchService: searchService,
            recentActivityStore: recentActivityStore,
            assetsEnabler: assetsEnabler,
            perpetualService: perpetualService,
            preferences: preferences,
            onDismissSearch: {},
            onSelectAssetAction: { _ in },
            onAddToken: {},
        )
    }
}
