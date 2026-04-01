// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import Primitives
import Style
import SwiftUI

public struct PriceViewModel: Sendable {
    public let price: Price?

    private let currencyFormatter: CurrencyFormatter
    static let percentFormatter = CurrencyFormatter.percent

    public init(
        price: Price?,
        currencyCode: String,
        currencyFormatterType: CurrencyFormatterType = .abbreviated,
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
        guard let value else { return Colors.gray }
        return PriceChangeColor.color(for: value)
    }

    public var priceChangeTextBackgroundColor: Color {
        if priceChange == 0 {
            return Colors.grayVeryLight
        } else if priceChange ?? 0 > 0 {
            return Colors.greenLight
        }
        return Colors.redLight
    }

    public func fiatAmountText(amount: Double) -> String {
        currencyFormatter.string(amount)
    }
}
