// Copyright (c). Gem Wallet. All rights reserved.

import Localization
import Primitives

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
