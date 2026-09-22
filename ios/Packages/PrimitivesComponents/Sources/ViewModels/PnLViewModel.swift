// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.formattedPercentage
import enum Gemstone.GemPercentageStyle
import class Gemstone.PriceChangeCalculator
import GemstonePrimitives
import Localization
import Primitives
import Style
import SwiftUI

public struct PnLViewModel {
    private let priceChangeCalculator = PriceChangeCalculator()
    private let pnl: Double?
    private let marginAmount: Double
    private let currencyFormatter: CurrencyFormatter
    private let percentageStyle: GemPercentageStyle

    public init(
        pnl: Double?,
        marginAmount: Double,
        currencyFormatter: CurrencyFormatter,
        percentageStyle: GemPercentageStyle,
    ) {
        self.pnl = pnl
        self.marginAmount = marginAmount
        self.currencyFormatter = currencyFormatter
        self.percentageStyle = percentageStyle
    }

    public var title: String {
        Localized.Perpetual.pnl
    }

    private var valueChange: PriceChangeViewModel {
        PriceChangeViewModel(value: pnl, currencyFormatter: currencyFormatter)
    }

    public var text: String? {
        guard let amountText = valueChange.text else { return nil }
        return priceChangeCalculator.pnlText(formattedAmount: amountText, formattedPercentage: Gemstone.formattedPercentage(value: percent, style: percentageStyle).text())
    }

    public var percent: Double {
        priceChangeCalculator.pnlPercentage(pnl: pnl ?? .zero, margin: marginAmount)
    }

    public var color: Color {
        valueChange.color
    }

    public var textStyle: TextStyle {
        valueChange.textStyle
    }
}
