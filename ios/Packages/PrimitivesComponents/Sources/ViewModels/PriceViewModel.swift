// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import class Gemstone.CryptoFiatConverter
import enum Gemstone.GemCurrencyStyle
import Components
import Formatters
import Foundation
import GemstonePrimitives
import Primitives
import Style
import SwiftUI
import func Gemstone.valueTone

public struct PriceViewModel: Sendable {
    public let price: Price?

    private let currencyFormatter: CurrencyFormatter
    static let percentFormatter = PercentFormatter.signed

    public init(
        price: Price?,
        currencyCode: String,
        currencyFormatterType: GemCurrencyStyle = .currency,
    ) {
        self.price = price
        currencyFormatter = CurrencyFormatter(type: currencyFormatterType, currencyCode: currencyCode)
    }

    public var isPriceAvailable: Bool {
        guard let price else { return false }
        return price.price != 0
    }

    public var priceAmountText: String {
        guard let price else { return "" }
        return currencyFormatter.string(price.price)
    }

    private var priceChange: Double? {
        price?.priceChangePercentage24h ?? .none
    }

    public var priceChangeText: String {
        guard let priceChange else { return "" }
        return Self.percentFormatter.string(priceChange)
    }

    public var priceChangeTextColor: Color {
        Self.priceChangeTextColor(value: priceChange)
    }

    public static func priceChangeTextColor(value: Double?) -> Color {
        valueTone(value: value ?? 0).color
    }

    public var priceChangeTextBackgroundColor: Color {
        valueTone(value: priceChange ?? 0).backgroundColor
    }

    public func fiatAmountText(amount: Double) -> String {
        currencyFormatter.string(amount)
    }

    public func fiatValueText(value: BigInt, decimals: Int) -> String? {
        guard let price, price.price != 0, value > 0 else { return nil }
        let amount = CryptoFiatConverter().toFiat(value: value, decimals: UInt32(decimals), price: price.price)
        return currencyFormatter.string(Double(amount) ?? .zero)
    }
}
