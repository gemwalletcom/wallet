// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemSwapPairSelection
import struct Gemstone.GemSwapPairSuggestion
import Primitives

extension Gemstone.GemSwapPairSuggestion {
    func map() -> SwapPairSelectorViewModel {
        SwapPairSelectorViewModel(
            fromAssetId: AssetId(core: payAssetId),
            toAssetId: receiveAssetId.map { AssetId(core: $0) },
        )
    }
}

public extension Gemstone.GemSwapPairSelection {
    func map() -> SwapPairSelectorViewModel {
        SwapPairSelectorViewModel(
            fromAssetId: payAssetId.map { AssetId(core: $0) },
            toAssetId: receiveAssetId.map { AssetId(core: $0) },
        )
    }
}
