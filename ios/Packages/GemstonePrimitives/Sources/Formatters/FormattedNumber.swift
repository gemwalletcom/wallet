// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import struct Gemstone.GemFormattedNumber
import enum Gemstone.GemNumberDisplay
import enum Gemstone.GemNumberNotation
import enum Gemstone.GemNumberRounding
import enum Gemstone.GemNumberUnit
import enum Gemstone.GemPrecision

public extension GemFormattedNumber {
    func text(locale: Locale = .current) -> String {
        switch notation {
        case .parenthesised: "(\(body(locale: locale)))"
        case .plain, .signed: body(locale: locale)
        }
    }
}

// MARK: - Private

private extension GemFormattedNumber {
    func body(locale: Locale) -> String {
        switch display {
        case let .number(precision):
            appendingSymbol(numberText(precision: precision, locale: locale))
        case .abbreviated:
            abbreviatedText(locale: locale)
        case let .belowThreshold(threshold, places):
            appendingSymbol("\(signText)<\(thresholdText(threshold, places: places, locale: locale))")
        }
    }

    var currencyCode: String? {
        switch unit {
        case let .currency(code): code
        case .percent, .symbol, .plain, .multiplier: nil
        }
    }

    var symbol: String? {
        switch unit {
        case let .symbol(symbol): symbol
        case .currency, .percent, .plain, .multiplier: nil
        }
    }

    var isPercent: Bool {
        switch unit {
        case .percent: true
        case .currency, .symbol, .plain, .multiplier: false
        }
    }

    var signText: String {
        guard showsSign else { return "" }
        return value < 0 ? "-" : "+"
    }

    var showsSign: Bool {
        switch notation {
        case .signed: true
        case .plain, .parenthesised: false
        }
    }

    var roundingRule: FloatingPointRoundingRule {
        switch rounding {
        case .toNearest: .toNearestOrEven
        case .towardZero: .towardZero
        }
    }

    var numberSign: NumberFormatStyleConfiguration.SignDisplayStrategy {
        showsSign ? .always(includingZero: true) : .automatic
    }

    var currencySign: CurrencyFormatStyleConfiguration.SignDisplayStrategy {
        showsSign ? .always(showZero: true) : .automatic
    }

    func numberText(precision: GemPrecision, locale: Locale) -> String {
        if isPercent {
            return value.formatted(
                .percent.locale(locale)
                    .precision(precision.formatStyle)
                    .sign(strategy: showsSign ? .always(includingZero: true) : .never)
                    .scale(1)
                    .rounded(rule: roundingRule),
            )
        }
        guard let currencyCode else {
            return value.formatted(.number.locale(locale).precision(precision.formatStyle).sign(strategy: numberSign).rounded(rule: roundingRule))
        }
        return value.formatted(.currency(code: currencyCode).locale(locale).precision(precision.formatStyle).sign(strategy: currencySign).rounded(rule: roundingRule))
    }

    func abbreviatedText(locale: Locale) -> String {
        let formatter = AbbreviatedFormatter(locale: locale)
        if let currencyCode {
            return formatter.string(from: value, currency: currencyCode, rule: roundingRule) ?? numberText(precision: .fraction(min: 2, max: 2), locale: locale)
        }
        return appendingSymbol(formatter.string(from: value, rule: roundingRule) ?? numberText(precision: .fraction(min: 2, max: 2), locale: locale))
    }

    func thresholdText(_ threshold: Double, places: UInt32, locale: Locale) -> String {
        guard let currencyCode else {
            return threshold.formatted(.number.locale(locale).precision(.fractionLength(Int(places))))
        }
        return threshold.formatted(.currency(code: currencyCode).locale(locale).precision(.fractionLength(Int(places))))
    }

    func appendingSymbol(_ text: String) -> String {
        if case .multiplier = unit {
            return "\(text)x"
        }
        guard let symbol else { return text }
        return "\(text) \(symbol)"
    }
}
