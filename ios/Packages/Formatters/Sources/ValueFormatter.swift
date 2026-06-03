// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Primitives

public struct ValueFormatter: Sendable {
    public enum Style: Sendable {
        case full, short, auto
    }

    private static let smallAmountThreshold = Decimal(sign: .plus, exponent: -1, significand: 1)
    private static let dustThreshold = Decimal(sign: .plus, exponent: -4, significand: 1)

    private let locale: Locale
    private let style: Style
    private let abbreviationThreshold: Decimal

    public init(
        locale: Locale = .current,
        style: Style,
        abbreviationThreshold: Decimal = defaultAbbreviationThreshold,
    ) {
        self.locale = locale
        self.style = style
        self.abbreviationThreshold = abbreviationThreshold
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
        if style == .short, abs(decimal) >= abbreviationThreshold, let abbreviated = abbreviatedFormatter.string(from: decimal) {
            return appendingCurrency(abbreviated, currency: currency)
        }
        if style == .short, abs(decimal) < Self.dustThreshold {
            return appendingCurrency("<\(formattedDustThreshold)", currency: currency)
        }
        return appendingCurrency(decimal.formatted(formatStyle(for: decimal)), currency: currency)
    }

    public func inputNumber(from string: String, decimals: Int) throws -> BigInt {
        try BigNumberFormatter.standard.number(
            from: NumberInputNormalizer.normalize(string, locale: locale),
            decimals: decimals,
        )
    }

    public func double(from number: BigInt, decimals: Int) throws -> Double {
        guard let result = BigNumberFormatter.standard.double(from: number, decimals: decimals) else {
            throw AnyError("unknown \(number) number")
        }
        return result
    }

    func number(amount: String) throws -> Decimal {
        try number(amount: amount, locale: locale)
    }

    func number(amount: String, locale: Locale) throws -> Decimal {
        guard let decimal = Decimal(string: amount, locale: locale) else {
            throw AnyError("unknown \(amount) decimal")
        }
        return decimal
    }
}

private extension ValueFormatter {
    var abbreviatedFormatter: AbbreviatedFormatter {
        AbbreviatedFormatter(locale: locale, threshold: abbreviationThreshold)
    }

    var formattedDustThreshold: String {
        Self.dustThreshold.formatted(
            Decimal.FormatStyle()
                .locale(locale)
                .precision(.fractionLength(4)),
        )
    }

    func formatStyle(for decimal: Decimal) -> Decimal.FormatStyle {
        Decimal.FormatStyle()
            .locale(locale)
            .grouping(.automatic)
            .rounded(rule: .towardZero)
            .precision(precision(for: abs(decimal)))
    }

    func precision(for magnitude: Decimal) -> NumberFormatStyleConfiguration.Precision {
        switch (style, magnitude) {
        case (.full, _): .full
        case (.short, Self.smallAmountThreshold...): .upToTwoPlaces
        case (.short, _): .upToFourPlaces
        case (.auto, 1...): .upToTwoPlaces
        case (.auto, Self.dustThreshold...): .fourSignificant
        case (.auto, _): .full
        }
    }

    func appendingCurrency(_ value: String, currency: String) -> String {
        currency.isEmpty ? value : "\(value) \(currency)"
    }
}
