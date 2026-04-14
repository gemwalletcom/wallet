// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PerpetualServiceTestKit
import Preferences
import PreferencesTestKit
import PriceService
import PriceServiceTestKit
import Primitives
import PrimitivesTestKit
import WalletTab

public extension PortfolioDataService {
    static func mock(
        portfolioService: PortfolioService = .mock(),
        perpetualService: PerpetualServiceMock = PerpetualServiceMock(),
        priceService: PriceService = .mock()
    ) -> PortfolioDataService {
        PortfolioDataService(
            portfolioService: portfolioService,
            perpetualService: perpetualService,
            priceService: priceService,
        )
    }
}

public extension PortfolioSceneViewModel {
    @MainActor
    static func mock(
        wallet: Wallet = .mock(),
        service: PortfolioDataService = .mock(),
        preferences: ObservablePreferences = .mock(),
        defaultType: PortfolioType = .wallet
    ) -> PortfolioSceneViewModel {
        PortfolioSceneViewModel(
            wallet: wallet,
            service: service,
            preferences: preferences,
            defaultType: defaultType,
        )
    }
}
