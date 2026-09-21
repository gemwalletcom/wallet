// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import Foundation
import func Gemstone.dustThreshold
import func Gemstone.dustThresholdPlaces
import enum Gemstone.GemValueStyle
import Primitives

public struct ValueFormatter: Sendable {
    private let locale: Locale
    private let style: GemValueStyle

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
        let magnitude = decimal.doubleValue
        if style.abbreviates(magnitude: magnitude), let abbreviated = abbreviatedFormatter.string(from: decimal) {
            return appendingCurrency(abbreviated, currency: currency)
        }
        if style.isDust(magnitude: magnitude) {
            return appendingCurrency("<\(formattedDustThreshold)", currency: currency)
        }
        return appendingCurrency(decimal.formatted(formatStyle(for: magnitude)), currency: currency)
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
    var abbreviatedFormatter: AbbreviatedFormatter {
        AbbreviatedFormatter(locale: locale)
    }

    var formattedDustThreshold: String {
        Decimal(dustThreshold()).formatted(
            Decimal.FormatStyle()
                .locale(locale)
                .precision(.fractionLength(Int(dustThresholdPlaces()))),
        )
    }

    func formatStyle(for magnitude: Double) -> Decimal.FormatStyle {
        Decimal.FormatStyle()
            .locale(locale)
            .grouping(.automatic)
            .rounded(rule: .towardZero)
            .precision(style.precision(magnitude: magnitude).formatStyle)
    }

    func appendingCurrency(_ value: String, currency: String) -> String {
        currency.isEmpty ? value : "\(value) \(currency)"
    }
}
