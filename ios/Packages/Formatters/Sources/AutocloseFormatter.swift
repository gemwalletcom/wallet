// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public struct AutocloseFormatter: Sendable {
    private let currencyFormatter: CurrencyFormatter
    private let takeProfitLabel: String
    private let stopLossLabel: String

    public init(
        currencyFormatter: CurrencyFormatter = CurrencyFormatter(type: .currency, currencyCode: Currency.usd.rawValue),
        takeProfitLabel: String,
        stopLossLabel: String,
    ) {
        self.currencyFormatter = currencyFormatter
        self.takeProfitLabel = takeProfitLabel
        self.stopLossLabel = stopLossLabel
    }

    public func format(
        takeProfit: Double?,
        stopLoss: Double?,
        takeProfitCanceled: Bool = false,
        stopLossCanceled: Bool = false,
    ) -> (subtitle: String, subtitleExtra: String?) {
        let tp: String? = {
            if takeProfitCanceled { return "\(takeProfitLabel): -" }
            return takeProfit.map { "\(takeProfitLabel): \(currencyFormatter.string($0))" }
        }()

        let sl: String? = {
            if stopLossCanceled { return "\(stopLossLabel): -" }
            return stopLoss.map { "\(stopLossLabel): \(currencyFormatter.string($0))" }
        }()

        switch (tp, sl) {
        case let (.some(tpText), .some(slText)):
            return (subtitle: tpText, subtitleExtra: slText)
        case let (.some(tpText), .none):
            return (subtitle: tpText, subtitleExtra: nil)
        case let (.none, .some(slText)):
            return (subtitle: slText, subtitleExtra: nil)
        case (.none, .none):
            return (subtitle: "-", subtitleExtra: nil)
        }
    }
}
