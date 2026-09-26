// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemSlippageSelection
import Primitives
import Swap

public extension SwapSlippageSceneViewModel {
    static func mock(
        slippage: GemSlippageSelection = .auto,
        onSelect: @escaping (GemSlippageSelection) -> Void = { _ in },
    ) -> SwapSlippageSceneViewModel {
        SwapSlippageSceneViewModel(chain: .ethereum, slippage: slippage, onSelect: onSelect)
    }
}
