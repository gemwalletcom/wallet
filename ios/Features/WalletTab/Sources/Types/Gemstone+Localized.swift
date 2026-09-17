// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.PortfolioStatistic
import Localization
import Primitives

extension PortfolioStatistic {
    var title: String {
        switch self {
        case .allTimeHigh: Localized.Asset.allTimeHigh
        case .allTimeLow: Localized.Asset.allTimeLow
        case .unrealizedPnl: Localized.Perpetual.unrealizedPnl
        case .accountLeverage: Localized.Perpetual.accountLeverage
        case .marginUsage: Localized.Perpetual.marginUsage
        case .allTimePnl: Localized.Perpetual.allTimePnl
        case .volume: Localized.Perpetual.volume
        }
    }
}

extension PortfolioType {
    var title: String {
        switch self {
        case .wallet: Localized.Wallet.Portfolio.title
        case .perpetuals: Localized.Perpetuals.title
        }
    }
}

extension PortfolioChartType {
    var title: String {
        switch self {
        case .value: Localized.Perpetual.value
        case .pnl: Localized.Perpetual.pnl
        }
    }
}
