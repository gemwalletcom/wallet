// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemPerpetualPortfolio {
    func map() -> PerpetualPortfolio {
        PerpetualPortfolio(
            day: day?.map(),
            week: week?.map(),
            month: month?.map(),
            allTime: allTime?.map(),
            accountSummary: accountSummary?.map(),
        )
    }
}

public extension GemPerpetualPortfolioTimeframeData {
    func map() -> PerpetualPortfolioTimeframeData {
        PerpetualPortfolioTimeframeData(
            accountValueHistory: accountValueHistory.map { $0.map() },
            pnlHistory: pnlHistory.map { $0.map() },
            volume: volume,
        )
    }
}

public extension GemPerpetualAccountSummary {
    func map() -> PerpetualAccountSummary {
        PerpetualAccountSummary(
            accountValue: accountValue,
            accountLeverage: accountLeverage,
            marginUsage: marginUsage,
            unrealizedPnl: unrealizedPnl,
        )
    }
}
