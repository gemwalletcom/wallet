// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemSwapQuoteInput

struct SwapLoadTrigger: DebouncableTrigger {
    let input: GemSwapQuoteInput
    let isImmediate: Bool
}
