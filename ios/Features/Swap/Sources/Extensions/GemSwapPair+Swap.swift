// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemSwapPairSelection
import struct Gemstone.GemSwapPairSuggestion
import Primitives

extension Gemstone.GemSwapPairSuggestion {
    func map() -> SwapPairSelectorViewModel {
        SwapPairSelectorViewModel(
            fromAssetId: payAssetId,
            toAssetId: receiveAssetId,
        )
    }
}

public extension Gemstone.GemSwapPairSelection {
    func map() -> SwapPairSelectorViewModel {
        SwapPairSelectorViewModel(
            fromAssetId: payAssetId,
            toAssetId: receiveAssetId,
        )
    }
}
