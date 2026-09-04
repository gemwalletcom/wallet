// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.SwapProvider
import struct Gemstone.SwapperProviderType

public extension SwapperProviderType {
    static func mock(id: SwapProvider = .pancakeswapV3) -> SwapperProviderType {
        SwapperProviderType(
            id: id,
            name: "PancakeSwap",
            protocol: "v3",
            protocolId: "pancakeswap_v3",
            mode: .onChain,
            slippageMode: .exact,
        )
    }
}
