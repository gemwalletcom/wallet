// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct PercentFormatter: Sendable, Hashable {
    private let locale: Locale
    private let fractionLength: ClosedRange<Int>
    private let showsSign: Bool

    public init(locale: Locale = .current, fractionLength: ClosedRange<Int> = 2 ... 2, showsSign: Bool = true) {
        self.locale = locale
        self.fractionLength = fractionLength
        self.showsSign = showsSign
    }

    public func string(_ value: Double) -> String {
        (value / 100).formatted(
            .percent
                .locale(locale)
                .precision(.fractionLength(fractionLength))
                .sign(strategy: showsSign ? .always(includingZero: true) : .never),
        )
    }
}
