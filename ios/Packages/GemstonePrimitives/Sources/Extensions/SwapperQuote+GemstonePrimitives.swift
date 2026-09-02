// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import struct Gemstone.GemSwapTransfer
import struct Gemstone.SwapperQuote
import class Gemstone.GemSwapQuoteSummary
import Primitives
import struct Gemstone.GemRecipient
import struct Gemstone.GemTransferData

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

public extension GemTransferData {
    init(swap transfer: GemSwapTransfer, fromAsset: Asset, toAsset: Asset) throws {
        let quote = try Primitives.SwapQuote(transfer.quote)
        self.init(
            inputType: .swap(
                fromAsset: fromAsset.map(),
                toAsset: toAsset.map(),
                swapData: SwapData(quote: quote, data: try Primitives.SwapQuoteData(transfer.data)).json(),
            ),
            recipient: GemRecipient(address: transfer.recipient),
            value: transfer.value,
            useMaxAmount: transfer.useMaxAmount,
            minimumValue: quote.minFromValue,
        )
    }
}
