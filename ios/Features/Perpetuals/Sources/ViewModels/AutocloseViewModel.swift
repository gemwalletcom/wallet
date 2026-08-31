// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI
import class Gemstone.GemAutocloseEstimator

public struct AutocloseViewModel {
    private let type: TpslType
    private let price: Double?
    private let estimator: GemAutocloseEstimator
    private let currencyFormatter: CurrencyFormatter
    private let percentFormatter: PercentFormatter

    public init(
        type: TpslType,
        price: Double?,
        estimator: GemAutocloseEstimator,
        currencyFormatter: CurrencyFormatter,
        percentFormatter: PercentFormatter,
    ) {
        self.type = type
        self.price = price
        self.estimator = estimator
        self.currencyFormatter = currencyFormatter
        self.percentFormatter = percentFormatter
    }

    public var priceTitle: String {
        Localized.Asset.price
    }

    public var title: String {
        switch type {
        case .takeProfit: Localized.Perpetual.AutoClose.takeProfit
        case .stopLoss: Localized.Perpetual.AutoClose.stopLoss
        }
    }

    public var profitTitle: String {
        let isProfit = price.map { estimator.pnl(price: $0) >= 0 } ?? (type == .takeProfit)
        return isProfit ? Localized.Perpetual.AutoClose.expectedProfit : Localized.Perpetual.AutoClose.expectedLoss
    }

    public var expectedPnL: String {
        guard let price else { return "-" }
        let pnl = estimator.pnl(price: price)
        let roe = estimator.roe(price: price)
        let percentText = percentFormatter.string(roe)

        guard estimator.hasSize() else {
            return percentText
        }

        let amount = currencyFormatter.string(abs(pnl))
        let sign = pnl >= 0 ? "+" : "-"
        return "\(sign)\(amount) (\(percentText))"
    }

    public var roeColor: Color {
        guard let price else { return Colors.secondaryText }
        let roe = estimator.roe(price: price)
        return PriceChangeColor.color(for: roe)
    }

    public var percents: [Int] {
        estimator.percentSuggestions().map { Int($0) }
    }

    public var percentSuggestions: [PercentageSuggestion] {
        percents.map { PercentageSuggestion(value: $0) }
    }
}
