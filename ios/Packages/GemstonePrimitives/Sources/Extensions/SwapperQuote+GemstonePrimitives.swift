// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import struct Gemstone.GemSwapTransfer
import struct Gemstone.SwapperQuote
import class Gemstone.GemSwapQuoteService
import Primitives

public extension Gemstone.SwapperQuote {
    func map(swapQuoteService: GemSwapQuoteService) throws -> Primitives.SwapQuote {
        try Primitives.SwapQuote(swapQuoteService.quote(quote: self))
    }

    var toValueBigInt: BigInt {
        (try? BigInt.from(string: toValue)) ?? .zero
    }

    var fromValueBigInt: BigInt {
        (try? BigInt.from(string: fromValue)) ?? .zero
    }
}

public extension TransferData {
    init(swap transfer: GemSwapTransfer, fromAsset: Asset, toAsset: Asset) throws {
        let quote = try Primitives.SwapQuote(transfer.quote)
        let value = try BigInt.from(string: transfer.value)
        self.init(
            type: .swap(fromAsset, toAsset, SwapData(quote: quote, data: try Primitives.SwapQuoteData(transfer.data))),
            recipient: Recipient(name: .none, address: transfer.recipient, memo: .none),
            value: value,
            useMaxAmount: transfer.useMaxAmount,
            minimumValue: quote.minFromValueBigInt,
        )
    }
}
