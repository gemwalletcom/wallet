// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import struct Gemstone.GemSwapRate

public struct AssetRateViewModel {
    private let rate: GemSwapRate
    private let locale: Locale

    public init(rate: GemSwapRate, locale: Locale = .current) {
        self.rate = rate
        self.locale = locale
    }

    public func text(isInverse: Bool) -> String {
        let rate = isInverse ? rate.inverse : rate.direct
        return rate.text(formattedValue: rate.value.text(locale: locale))
    }
}
