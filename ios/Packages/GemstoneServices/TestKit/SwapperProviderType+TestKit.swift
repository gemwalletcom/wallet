// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.SwapperProviderType
import enum Gemstone.SwapProvider

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
