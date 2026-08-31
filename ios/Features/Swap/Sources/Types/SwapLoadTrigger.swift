// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation

struct SwapLoadTrigger: DebouncableTrigger {
    let id = UUID()
    let input: SwapQuoteInput
    let isImmediate: Bool
}
