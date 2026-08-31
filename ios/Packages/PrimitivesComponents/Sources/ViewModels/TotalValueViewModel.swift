// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import GemstonePrimitives
import Primitives
import Style
import SwiftUI

public struct TotalValueViewModel {
    private let totalValue: TotalFiatValue
    private let currencyFormatter: CurrencyFormatter

    public init(totalValue: TotalFiatValue, currencyFormatter: CurrencyFormatter) {
        self.totalValue = totalValue
        self.currencyFormatter = currencyFormatter
    }

    public var title: String {
        currencyFormatter.string(totalValue.value)
    }

    public var pnlAmountText: String? {
        guard showsPnl else { return nil }
        return PriceChangeViewModel(value: totalValue.pnlAmount, currencyFormatter: currencyFormatter).text
    }

    public var pnlPercentageText: String? {
        guard showsPnl else { return nil }
        return PercentFormatter.unsigned.string(totalValue.pnlPercentage)
    }

    private var showsPnl: Bool {
        totalValue.showsPnl
    }

    public var pnlColor: Color {
        PriceChangeColor.color(for: totalValue.pnlAmount)
    }
}
