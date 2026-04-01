// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.SwapperMode
import struct Gemstone.SwapperOptions
import struct Gemstone.SwapperQuoteAsset
import struct Gemstone.SwapperQuoteRequest

extension SwapperQuoteRequest {
    static func mock() -> SwapperQuoteRequest {
        SwapperQuoteRequest(
            fromAsset: .mock(),
            toAsset: .mockUSDT(),
            walletAddress: "0x",
            destinationAddress: "0x",
            value: "1000000000000000000",
            mode: .exactIn,
            options: .mock(),
        )
    }
}
