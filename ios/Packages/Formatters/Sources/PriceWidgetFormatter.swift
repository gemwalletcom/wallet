// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct PriceWidgetFormatter: Sendable, Hashable {
    public enum Style: Sendable, Hashable {
        case plain, abbreviated
    }

    private let locale: Locale
    private let style: Style
    private let currencyCode: String

    public init(
        style: Style = .plain,
        locale: Locale = Locale.current,
        currencyCode: String,
    ) {
        self.style = style
        self.locale = locale
        self.currencyCode = currencyCode
    }

    public func string(_ value: Double) -> String {
        switch style {
        case .plain: currencyString(value)
        case .abbreviated: abbreviatedFormatter.string(from: value, currency: currencyCode) ?? currencyString(value)
        }
    }
}

// MARK: - Private

private extension PriceWidgetFormatter {
    static let smallValueThreshold: Double = 0.99
    static let dustThreshold: Double = 1e-10

    var abbreviatedFormatter: AbbreviatedFormatter {
        AbbreviatedFormatter(locale: locale)
    }

    func currencyString(_ value: Double) -> String {
        value.formatted(.currency(code: currencyCode).locale(locale).precision(precision(for: abs(value))))
    }

    func precision(for magnitude: Double) -> NumberFormatStyleConfiguration.Precision {
        switch magnitude {
        case Self.dustThreshold ..< Self.smallValueThreshold: .fourSignificant
        default: .twoPlaces
        }
    }
}
