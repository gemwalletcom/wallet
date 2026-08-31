// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemSwapPairSuggestion
import Primitives

extension Gemstone.GemSwapPairSuggestion {
    func map() throws -> SwapPairSelectorViewModel {
        try SwapPairSelectorViewModel(
            fromAssetId: AssetId(id: payAssetId),
            toAssetId: receiveAssetId.map { try AssetId(id: $0) },
        )
    }
}
