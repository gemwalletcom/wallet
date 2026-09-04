// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Primitives
import Style
import SwiftUI

public struct TotalValueViewModel {
    private let totalValue: TotalFiatValue
    private let currencyFormatter: CurrencyFormatter
    private let showsPnl: Bool

    public init(totalValue: TotalFiatValue, currencyFormatter: CurrencyFormatter, showsPnl: Bool) {
        self.totalValue = totalValue
        self.currencyFormatter = currencyFormatter
        self.showsPnl = showsPnl
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

    public var pnlColor: Color {
        PriceChangeColor.color(for: totalValue.pnlAmount)
    }
}
