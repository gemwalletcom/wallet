// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import PriceService
import Primitives
import Store
import StoreTestKit

public struct GemAPIPortfolioServiceMock: GemAPIPortfolioService {
    private let allTimeHigh: ChartValuePercentage?
    private let allTimeLow: ChartValuePercentage?

    public init(allTimeHigh: ChartValuePercentage? = nil, allTimeLow: ChartValuePercentage? = nil) {
        self.allTimeHigh = allTimeHigh
        self.allTimeLow = allTimeLow
    }

    public func getPortfolioAssets(period _: ChartPeriod, request _: PortfolioAssetsRequest) async throws -> PortfolioAssets {
        PortfolioAssets(totalValue: 0, values: [], allTimeHigh: allTimeHigh, allTimeLow: allTimeLow, allocation: [])
    }
}

public extension PortfolioService {
    static func mock(
        apiService: any GemAPIPortfolioService = GemAPIPortfolioServiceMock(),
        assetStore: AssetStore = .mock(),
    ) -> PortfolioService {
        PortfolioService(
            apiService: apiService,
            assetStore: assetStore,
        )
    }
}
