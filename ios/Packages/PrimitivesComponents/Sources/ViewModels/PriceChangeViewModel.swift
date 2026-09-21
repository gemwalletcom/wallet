// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import class Gemstone.PriceChangeCalculator
import func Gemstone.valueTone
import GemstonePrimitives
import Style
import SwiftUI

public struct PriceChangeViewModel {
    private let value: Double?
    private let currencyFormatter: CurrencyFormatter
    private let priceChangeCalculator = PriceChangeCalculator()

    public init(value: Double?, currencyFormatter: CurrencyFormatter) {
        self.value = value
        self.currencyFormatter = currencyFormatter
    }

    public var text: String? {
        guard let value else { return nil }
        return priceChangeCalculator.sign(value: value).format(amount: currencyFormatter.string(abs(value)))
    }

    public var color: Color {
        guard let value else { return .secondary }
        return valueTone(value: value).color
    }

    public var textStyle: TextStyle {
        TextStyle(font: .callout, color: color)
    }
}
