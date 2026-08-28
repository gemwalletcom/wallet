// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives

struct AddAssetLoadTrigger: DebouncableTrigger {
    let chain: Chain
    let address: String
    let isImmediate: Bool
}
