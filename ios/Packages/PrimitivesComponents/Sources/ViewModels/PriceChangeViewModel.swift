// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import class Gemstone.PriceChangeCalculator
import Foundation
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
        return PriceChangeColor.color(for: value)
    }

    public var textStyle: TextStyle {
        TextStyle(font: .callout, color: color)
    }
}
