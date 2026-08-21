// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.SwapperProvider
import struct Gemstone.SwapperProviderData
import struct Gemstone.SwapperRoute

extension SwapperProviderData {
    static func mock(provider: SwapperProvider = .pancakeswapV3) -> SwapperProviderData {
        SwapperProviderData(
            provider: .mock(id: provider),
            slippageBps: 50,
            routes: [.mock()],
        )
    }
}
