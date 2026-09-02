// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import struct Gemstone.SwapperQuote
import class Gemstone.GemSwapQuoteSummary
import Primitives

public extension Gemstone.SwapperQuote {
    func map() throws -> Primitives.SwapQuote {
        try Primitives.SwapQuote(GemSwapQuoteSummary.fromQuote(quote: self).quote())
    }

    var toValueBigInt: BigInt {
        (try? BigInt.from(string: toValue)) ?? .zero
    }

    var fromValueBigInt: BigInt {
        (try? BigInt.from(string: fromValue)) ?? .zero
    }
}
