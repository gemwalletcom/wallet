// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemSlippageSelection
import Primitives
import Swap

public extension SwapSlippageViewModel {
    static func mock(
        slippage: GemSlippageSelection = .auto,
        onSelect: @escaping (GemSlippageSelection) -> Void = { _ in },
    ) -> SwapSlippageViewModel {
        SwapSlippageViewModel(chain: .ethereum, slippage: slippage, onSelect: onSelect)
    }
}
