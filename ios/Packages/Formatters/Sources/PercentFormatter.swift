// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct PercentFormatter: Sendable, Hashable {
    public enum Sign: Sendable {
        case signed, unsigned
    }

    public static let signed = PercentFormatter(sign: .signed)
    public static let unsigned = PercentFormatter(sign: .unsigned)

    private let locale: Locale
    private let sign: Sign

    public init(locale: Locale = .current, sign: Sign = .signed) {
        self.locale = locale
        self.sign = sign
    }

    public func string(_ value: Double) -> String {
        (value / 100).formatted(
            .percent
                .locale(locale)
                .precision(.fractionLength(2))
                .sign(strategy: signStrategy),
        )
    }

    private var signStrategy: NumberFormatStyleConfiguration.SignDisplayStrategy {
        switch sign {
        case .signed: .always(includingZero: true)
        case .unsigned: .never
        }
    }
}
