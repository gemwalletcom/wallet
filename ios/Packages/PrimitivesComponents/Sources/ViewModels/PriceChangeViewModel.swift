// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import func Gemstone.formattedSignedCurrency
import func Gemstone.valueTone
import GemstonePrimitives
import Style
import SwiftUI

public struct PriceChangeViewModel {
    private let value: Double?
    private let currencyFormatter: CurrencyFormatter

    public init(value: Double?, currencyFormatter: CurrencyFormatter) {
        self.value = value
        self.currencyFormatter = currencyFormatter
    }

    public var text: String? {
        guard let value else { return nil }
        return formattedSignedCurrency(value: value, code: currencyFormatter.currencyCode, style: currencyFormatter.type)
            .text(locale: currencyFormatter.locale)
    }

    public var color: Color {
        guard let value else { return .secondary }
        return valueTone(value: value).color
    }

    public var textStyle: TextStyle {
        TextStyle(font: .callout, color: color)
    }
}
