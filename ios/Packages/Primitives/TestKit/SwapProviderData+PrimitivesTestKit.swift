// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension SwapProviderData {
    static func mock(
        provider _: SwapProvider = .uniswapV4,
        name: String = "Uniswap",
        protocolName: String = "Uniswap v3",
    ) -> SwapProviderData {
        SwapProviderData(
            provider: .mayan,
            name: name,
            protocolName: protocolName,
        )
    }
}
