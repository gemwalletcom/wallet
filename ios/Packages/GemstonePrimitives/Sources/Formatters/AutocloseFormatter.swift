// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemPerpetual
import Primitives

public struct AutocloseFormatter: Sendable {
    private let currencyFormatter: CurrencyFormatter
    private let takeProfitLabel: String
    private let stopLossLabel: String
    private let perpetual = GemPerpetual(provider: .hypercore)

    public init(
        currencyFormatter: CurrencyFormatter = .usd,
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
        format(
            takeProfitText: takeProfit.map { currencyFormatter.string($0) },
            stopLossText: stopLoss.map { currencyFormatter.string($0) },
            takeProfitCanceled: takeProfitCanceled,
            stopLossCanceled: stopLossCanceled,
        )
    }

    public func format(
        takeProfitText: String?,
        stopLossText: String?,
        takeProfitCanceled: Bool = false,
        stopLossCanceled: Bool = false,
    ) -> (subtitle: String, subtitleExtra: String?) {
        let tp: String? = {
            if takeProfitCanceled { return perpetual.triggerOrderText(label: takeProfitLabel, formattedPrice: nil) }
            return takeProfitText.map { perpetual.triggerOrderText(label: takeProfitLabel, formattedPrice: $0) }
        }()

        let sl: String? = {
            if stopLossCanceled { return perpetual.triggerOrderText(label: stopLossLabel, formattedPrice: nil) }
            return stopLossText.map { perpetual.triggerOrderText(label: stopLossLabel, formattedPrice: $0) }
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
