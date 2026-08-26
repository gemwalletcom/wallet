// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneStore
import PerpetualService
import protocol Gemstone.GemPerpetualServiceProtocol
import GemstonePrimitivesTestKit
import Preferences
import PreferencesTestKit
import Primitives
import Store
import StoreTestKit

public extension PerpetualService {
    static func mock(
        db: DB = .mock(),
        provider: PerpetualProvidable = PerpetualProviderMock(),
        service: any GemPerpetualServiceProtocol = GemPerpetualServiceMock(),
        preferences: Preferences = .mock(),
    ) -> PerpetualService {
        PerpetualService(
            store: PerpetualStore(db: db),
            perpetualStore: GemstonePerpetualStore(store: PerpetualStore(db: db), assetStore: AssetStore(db: db), balanceStore: BalanceStore(db: db)),
            balanceStore: BalanceStore(db: db),
            provider: provider,
            service: service,
            preferences: preferences,
        )
    }
}

public struct PerpetualProviderMock: PerpetualProvidable {
    public init() {}

    public func provider() -> PerpetualProvider {
        .hypercore
    }

    public func getAccountMode(address _: String) async throws -> PerpetualAccountMode {
        .standard
    }

    public func getCandlesticks(symbol _: String, period _: ChartPeriod) async throws -> [ChartCandleStick] {
        []
    }

    public func getPortfolio(address _: String) async throws -> PerpetualPortfolio {
        PerpetualPortfolio(day: nil, week: nil, month: nil, allTime: nil, accountSummary: nil)
    }
}
