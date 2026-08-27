// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemWalletPreferencesServiceProtocol
import class Gemstone.GemWalletPreferencesService
import GemstoneServices
import protocol Gemstone.GemPerpetualServiceProtocol
import GemstonePrimitivesTestKit
import Preferences
import PreferencesTestKit
import Primitives
import Store
import StoreTestKit

public extension PerpetualService {
    static func mock(
        provider: PerpetualProvidable = PerpetualProviderMock(),
        service: any GemPerpetualServiceProtocol = GemPerpetualServiceMock(),
    ) -> PerpetualService {
        PerpetualService(provider: provider, service: service)
    }
}

public struct PerpetualProviderMock: PerpetualProvidable {
    public init() {}

    public func provider() -> PerpetualProvider {
        .hypercore
    }

    public func getCandlesticks(symbol _: String, period _: ChartPeriod) async throws -> [ChartCandleStick] {
        []
    }

    public func getPortfolio(address _: String) async throws -> PerpetualPortfolio {
        PerpetualPortfolio(day: nil, week: nil, month: nil, allTime: nil, accountSummary: nil)
    }
}
