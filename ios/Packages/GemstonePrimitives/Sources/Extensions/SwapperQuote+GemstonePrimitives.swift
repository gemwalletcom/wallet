// Copyright (c). Gem Wallet. All rights reserved.

import func Gemstone.swapperQuoteSummary
import struct Gemstone.SwapperQuote
import struct Gemstone.SwapQuote

public extension Gemstone.SwapperQuote {
    var swapQuote: Gemstone.SwapQuote {
        swapperQuoteSummary(quote: self).quote
    }
}
