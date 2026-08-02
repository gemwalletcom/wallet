// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import Primitives
import Style
import SwiftUI

public struct ChartHeaderViewModel {
    public let period: ChartPeriod
    public let date: Date?
    public let price: Double
    public let priceChangePercentage: Double
    public let headerValue: Double?
    public let type: ChartValueType

    private let formatter: CurrencyFormatter
    private let dateFormatter: ChartDateFormatter
    private let hideBalance: Bool

    public init(
        period: ChartPeriod,
        date: Date?,
        price: Double,
        priceChangePercentage: Double,
        headerValue: Double? = nil,
        formatter: CurrencyFormatter,
        dateFormatter: ChartDateFormatter = ChartDateFormatter(),
        type: ChartValueType = .price,
        hideBalance: Bool = false,
    ) {
        self.period = period
        self.date = date
        self.price = price
        self.priceChangePercentage = priceChangePercentage
        self.headerValue = headerValue
        self.type = type
        self.formatter = formatter
        self.dateFormatter = dateFormatter
        self.hideBalance = hideBalance
    }

    private var valueChange: PriceChangeViewModel? {
        type == .priceChange ? PriceChangeViewModel(value: price, currencyFormatter: formatter) : nil
    }

    public var dateText: String? {
        date.map { dateFormatter.string(for: $0, period: period) }
    }

    public var headerValueText: String? {
        guard let headerValue else { return nil }
        return formatter.string(headerValue).masked(if: hideBalance)
    }

    public var priceText: String {
        (valueChange?.text ?? formatter.string(price)).masked(if: hideBalance)
    }

    public var priceColor: Color {
        valueChange?.color ?? Colors.black
    }

    public var priceChangeText: String? {
        guard price != 0 else { return nil }
        switch type {
        case .priceChange:
            guard headerValue != nil, priceChangePercentage != 0 else { return nil }
            return "(\(PercentFormatter.unsigned.string(priceChangePercentage)))"
        case .price:
            return PercentFormatter.signed.string(priceChangePercentage)
        }
    }

    public var priceChangeTextColor: Color {
        PriceChangeColor.color(for: priceChangePercentage)
    }

    public var priceFont: Font {
        headerValue != nil ? .app.headline : .title2
    }

    public var priceChangeFont: Font {
        headerValue != nil ? .app.headline : .callout
    }
}
