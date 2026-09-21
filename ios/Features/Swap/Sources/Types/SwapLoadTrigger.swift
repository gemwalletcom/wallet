// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemSwapQuoteInput

struct SwapLoadTrigger: DebouncableTrigger {
    let id = UUID()
    let input: GemSwapQuoteInput
    let isImmediate: Bool
}
