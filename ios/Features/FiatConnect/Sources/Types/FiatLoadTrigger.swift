// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives

struct FiatLoadTrigger: DebouncableTrigger {
    let type: FiatQuoteType
    let amount: String
    let isImmediate: Bool
}
