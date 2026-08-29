// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemPortfolioServiceProtocol
import GemstonePrimitivesTestKit
import Preferences
import PreferencesTestKit
import Primitives
import PrimitivesTestKit
import WalletTab

public extension PortfolioSceneViewModel {
    @MainActor
    static func mock(
        wallet: Wallet = .mock(),
        portfolioService: any GemPortfolioServiceProtocol = GemPortfolioServiceMock(),
        preferences: ObservablePreferences = .mock(),
        defaultType: PortfolioType = .wallet,
    ) -> PortfolioSceneViewModel {
        PortfolioSceneViewModel(
            wallet: wallet,
            portfolioService: portfolioService,
            preferences: preferences,
            defaultType: defaultType,
        )
    }
}
