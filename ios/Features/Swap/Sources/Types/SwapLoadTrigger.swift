// Copyright (c). Gem Wallet. All rights reserved.

import Components

struct SwapLoadTrigger: DebouncableTrigger {
    let input: SwapQuoteInput
    let isImmediate: Bool
}
