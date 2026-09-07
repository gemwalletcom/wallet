// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemPortfolioServiceProtocol
import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
import WalletTab

public extension PortfolioSceneViewModel {
    @MainActor
    static func mock(
        wallet: Wallet = .mock(),
        service: any GemPortfolioServiceProtocol = GemPortfolioServiceMock(),
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
