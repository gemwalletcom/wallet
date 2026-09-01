// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemRecentActivityServiceProtocol
import class Gemstone.GemRecentActivityService
import protocol Gemstone.GemBalanceServiceProtocol
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
        recentAssetsService: any GemRecentActivityServiceProtocol = GemRecentActivityService(store: GemstoneRecentActivityStore(store: .mock())),
        balanceService: any GemBalanceServiceProtocol = .mock(),
        perpetualService: any GemPerpetualServiceProtocol = GemPerpetualServiceMock(),
        preferences: ObservablePreferences = .mock(),
    ) -> WalletSearchSceneViewModel {
        WalletSearchSceneViewModel(
            wallet: wallet,
            searchService: searchService,
            recentAssetsService: recentAssetsService,
            balanceService: balanceService,
            perpetualService: perpetualService,
            preferences: preferences,
            onDismissSearch: {},
            onSelectAssetAction: { _ in },
            onAddToken: {},
        )
    }
}
