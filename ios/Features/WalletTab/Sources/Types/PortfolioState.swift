// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives

struct PortfolioState {
    var wallet: StateViewType<PortfolioData> = .loading
    var perpetual: StateViewType<PortfolioData> = .loading
    var selectedType: PortfolioType = .wallet
    var selectedPeriod: ChartPeriod = .all
    var selectedChartType: PortfolioChartType = .pnl

    subscript(type: PortfolioType) -> StateViewType<PortfolioData> {
        get {
            switch type {
            case .wallet: wallet
            case .perpetuals: perpetual
            }
        }
        set {
            switch type {
            case .wallet: wallet = newValue
            case .perpetuals: perpetual = newValue
            }
        }
    }
}
