// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct PercentFormatter: Sendable, Hashable {
    public static let signed = PercentFormatter(signed: true)
    public static let unsigned = PercentFormatter(signed: false)

    private let locale: Locale
    private let signed: Bool

    public init(locale: Locale = .current, signed: Bool = true) {
        self.locale = locale
        self.signed = signed
    }

    public func string(_ value: Double) -> String {
        (value / 100).formatted(
            .percent
                .locale(locale)
                .precision(.fractionLength(2))
                .sign(strategy: signed ? .always(includingZero: true) : .never),
        )
    }
}
