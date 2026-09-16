// Copyright (c). Gem Wallet. All rights reserved.

import func Gemstone.swapQuote
import struct Gemstone.SwapperQuote
import struct Gemstone.SwapQuote

public extension Gemstone.SwapperQuote {
    var swapQuote: Gemstone.SwapQuote {
        Gemstone.swapQuote(quote: self)
    }
}
