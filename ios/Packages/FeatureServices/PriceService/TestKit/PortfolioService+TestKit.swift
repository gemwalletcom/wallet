// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import PriceService
import Primitives
import StoreTestKit

public struct GemAPIPortfolioServiceMock: GemAPIPortfolioService {
    public init() {}

    public func getPortfolioAssets(period _: ChartPeriod, request _: PortfolioAssetsRequest) async throws -> PortfolioAssets {
        PortfolioAssets(totalValue: 0, values: [], allTimeHigh: nil, allTimeLow: nil, allocation: [])
    }
}

public extension PortfolioService {
    static func mock() -> PortfolioService {
        PortfolioService(
            apiService: GemAPIPortfolioServiceMock(),
            assetStore: .mock(),
        )
    }
}
