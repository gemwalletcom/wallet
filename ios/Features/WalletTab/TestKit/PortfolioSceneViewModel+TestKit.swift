// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemPerpetualServiceProtocol
import StoreTestKit
import Store
import GemstonePrimitivesTestKit
import protocol Gemstone.GemPortfolioServiceProtocol
import Foundation
import GemstoneServicesTestKit
import Preferences
import PreferencesTestKit
import GemstoneServices
import Primitives
import PrimitivesTestKit
import WalletTab

public extension PortfolioDataService {
    static func mock(
        portfolioService: any GemPortfolioServiceProtocol = GemPortfolioServiceMock(),
    ) -> PortfolioDataService {
        PortfolioDataService(portfolioService: portfolioService)
    }
}

public extension PortfolioSceneViewModel {
    @MainActor
    static func mock(
        wallet: Wallet = .mock(),
        service: PortfolioDataService = .mock(),
        preferences: ObservablePreferences = .mock(),
        defaultType: PortfolioType = .wallet,
    ) -> PortfolioSceneViewModel {
        PortfolioSceneViewModel(
            wallet: wallet,
            service: service,
            preferences: preferences,
            defaultType: defaultType,
        )
    }
}
