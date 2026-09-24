// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import class Gemstone.CryptoFiatConverter
import enum Gemstone.GemCurrencyStyle
import func Gemstone.valueTone
import GemstonePrimitives
import Primitives
import Style
import SwiftUI

public struct PriceViewModel: Sendable {
    public let price: Price?

    private let currencyFormatter: CurrencyFormatter

    public init(
        price: Price?,
        currencyCode: String,
        currencyFormatterType: GemCurrencyStyle = .currency,
    ) {
        self.price = price
        currencyFormatter = CurrencyFormatter(type: currencyFormatterType, currencyCode: currencyCode)
    }

    public static func priceChangeTextColor(value: Double?) -> Color {
        valueTone(value: value ?? 0).color
    }

    public func fiatAmountText(amount: Double) -> String {
        currencyFormatter.string(amount)
    }
}
