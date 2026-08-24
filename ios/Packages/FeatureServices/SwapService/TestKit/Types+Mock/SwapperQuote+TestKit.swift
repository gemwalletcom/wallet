// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.SwapperProvider
import struct Gemstone.SwapperQuote

public extension SwapperQuote {
    static func mock(
        fromValue: String = "1000000000000000000",
        minFromValue: String? = nil,
        toValue: String = "250000000000",
        provider: SwapperProvider = .pancakeswapV3,
        etaInSeconds: UInt32? = nil,
    ) -> SwapperQuote {
        SwapperQuote(
            fromValue: fromValue,
            minFromValue: minFromValue,
            toValue: toValue,
            data: .mock(provider: provider),
            request: .mock(),
            etaInSeconds: etaInSeconds,
        )
    }
}
