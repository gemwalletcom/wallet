// Copyright (c). Gem Wallet. All rights reserved.

import ActivityService
import ActivityServiceTestKit
import AssetsService
import AssetsServiceTestKit
import BalanceService
import BalanceServiceTestKit
import PerpetualService
import PerpetualServiceTestKit
import Preferences
import PreferencesTestKit
import Primitives
import PrimitivesTestKit
import WalletTab

public extension WalletSearchSceneViewModel {
    @MainActor
    static func mock(
        wallet: Wallet = .mock(),
        searchService: WalletSearchService = .mock(),
        activityService: ActivityService = .mock(),
        assetsEnabler: any AssetsEnabler = .mock(),
        balanceService: BalanceService = .mock(),
        perpetualService: PerpetualService = .mock(),
        preferences: ObservablePreferences = .mock(),
    ) -> WalletSearchSceneViewModel {
        WalletSearchSceneViewModel(
            wallet: wallet,
            searchService: searchService,
            activityService: activityService,
            assetsEnabler: assetsEnabler,
            balanceService: balanceService,
            perpetualService: perpetualService,
            preferences: preferences,
            onDismissSearch: {},
            onSelectAssetAction: { _ in },
            onAddToken: {},
        )
    }
}
