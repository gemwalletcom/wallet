// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.SwapperQuote
import struct Gemstone.SwapQuote
import func Gemstone.swapQuote

public extension Gemstone.SwapperQuote {
    var swapQuote: Gemstone.SwapQuote {
        Gemstone.swapQuote(quote: self)
    }
}
