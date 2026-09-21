// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.SwapperProviderData
import struct Gemstone.SwapperRoute
import enum Gemstone.SwapProvider

extension SwapperProviderData {
    static func mock(provider: SwapProvider = .pancakeswapV3) -> SwapperProviderData {
        SwapperProviderData(
            provider: .mock(id: provider),
            slippageBps: 50,
            routes: [.mock()],
        )
    }
}
