// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemSwapQuoteSummary
import struct Gemstone.SwapperQuote
import struct Gemstone.SwapQuote

public extension Gemstone.SwapperQuote {
    var swapQuote: Gemstone.SwapQuote {
        GemSwapQuoteSummary.fromQuote(quote: self).quote()
    }
}
