// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Formatters
import struct Gemstone.GemFormattedNumber
import enum Gemstone.GemNumberDisplay
import enum Gemstone.GemNumberNotation
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
            appendingSymbol("<\(thresholdText(threshold, places: places, locale: locale))")
        }
    }

    var currencyCode: String? {
        switch unit {
        case let .currency(code): code
        case .percent, .symbol, .plain: nil
        }
    }

    var symbol: String? {
        switch unit {
        case let .symbol(symbol): symbol
        case .currency, .percent, .plain: nil
        }
    }

    var isPercent: Bool {
        switch unit {
        case .percent: true
        case .currency, .symbol, .plain: false
        }
    }

    var showsSign: Bool {
        switch notation {
        case .signed: true
        case .plain, .parenthesised: false
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
                    .scale(1),
            )
        }
        guard let currencyCode else {
            return value.formatted(.number.locale(locale).precision(precision.formatStyle).sign(strategy: numberSign))
        }
        return value.formatted(.currency(code: currencyCode).locale(locale).precision(precision.formatStyle).sign(strategy: currencySign))
    }

    func abbreviatedText(locale: Locale) -> String {
        let formatter = AbbreviatedFormatter(locale: locale)
        if let currencyCode {
            return formatter.string(from: value, currency: currencyCode) ?? numberText(precision: .fraction(min: 2, max: 2), locale: locale)
        }
        return appendingSymbol(formatter.string(from: value) ?? numberText(precision: .fraction(min: 2, max: 2), locale: locale))
    }

    func thresholdText(_ threshold: Double, places: UInt32, locale: Locale) -> String {
        guard let currencyCode else {
            return threshold.formatted(.number.locale(locale).precision(.fractionLength(Int(places))))
        }
        return threshold.formatted(.currency(code: currencyCode).locale(locale).precision(.fractionLength(Int(places))))
    }

    func appendingSymbol(_ text: String) -> String {
        guard let symbol else { return text }
        return "\(text) \(symbol)"
    }
}
