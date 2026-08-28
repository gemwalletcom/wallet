// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemPreferencesServiceProtocol
import Primitives

public extension GemPreferencesServiceProtocol {
    var currencyValue: Primitives.Currency {
        (try? Primitives.Currency(getCurrency())) ?? .usd
    }

    var currencyCode: String {
        currencyValue.rawValue
    }

    var chartPeriodValue: ChartPeriod {
        (try? ChartPeriod(getChartPeriod())) ?? .day
    }

    func setChartPeriodValue(_ period: ChartPeriod) {
        try? setChartPeriod(period: period.json())
    }

    var perpetualChartPeriodValue: ChartPeriod {
        (try? ChartPeriod(getPerpetualChartPeriod())) ?? .day
    }

    func setPerpetualChartPeriodValue(_ period: ChartPeriod) {
        try? setPerpetualChartPeriod(period: period.json())
    }

    var swapSlippage: SwapSlippage {
        switch getSwapSlippageBps() {
        case let .some(bps): .manual(bps: bps)
        case .none: .auto
        }
    }

    func setSwapSlippage(_ slippage: SwapSlippage) throws {
        switch slippage {
        case .auto: try setSwapSlippageBps(bps: nil)
        case let .manual(bps): try setSwapSlippageBps(bps: bps)
        }
    }

    var appearanceValue: Primitives.Appearance {
        (try? Primitives.Appearance(getAppearance())) ?? .system
    }

    func setAppearanceValue(_ appearance: Primitives.Appearance) throws {
        try setAppearance(appearance: appearance.json())
    }

    func showPerpetuals(for wallet: Wallet) -> Bool {
        (try? showPerpetuals(wallet: wallet.json())) ?? false
    }
}
