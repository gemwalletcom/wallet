// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import Foundation
import func Gemstone.formattedAmount
import enum Gemstone.GemPrecision
import enum Gemstone.GemValueStyle
import Primitives

public struct ValueFormatter: Sendable {
    public let locale: Locale
    public let style: GemValueStyle

    public init(
        locale: Locale = .current,
        style: GemValueStyle,
    ) {
        self.locale = locale
        self.style = style
    }

    public func string(_ value: BigInt, asset: Asset) -> String {
        string(value, decimals: asset.decimals.asInt, currency: asset.symbol)
    }

    public func string(_ value: BigInt, decimals: Int, currency: String = "") -> String {
        guard let decimal = BigNumberFormatter.standard.decimal(from: value, decimals: decimals) else {
            return ""
        }
        if value.isZero {
            return appendingCurrency("0", currency: currency)
        }
        let number = formattedAmount(value: decimal.doubleValue, symbol: currency.isEmpty ? nil : currency, style: style)
        guard case let .number(precision) = number.display else {
            return number.text(locale: locale)
        }
        return appendingCurrency(decimal.formatted(formatStyle(precision: precision)), currency: currency)
    }

    public func double(from number: BigInt, decimals: Int) throws -> Double {
        guard let result = BigNumberFormatter.standard.double(from: number, decimals: decimals) else {
            throw AnyError("unknown \(number) number")
        }
        return result
    }
}

// MARK: - Private

private extension ValueFormatter {
    func formatStyle(precision: GemPrecision) -> Decimal.FormatStyle {
        Decimal.FormatStyle()
            .locale(locale)
            .grouping(.automatic)
            .rounded(rule: .towardZero)
            .precision(precision.formatStyle)
    }

    func appendingCurrency(_ value: String, currency: String) -> String {
        currency.isEmpty ? value : "\(value) \(currency)"
    }
}
