// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import enum Gemstone.GemFiatAmountCheck
import struct Gemstone.FiatQuote
import struct Gemstone.FiatQuoteUrl
import protocol Gemstone.GemFiatQuoteServiceProtocol
import struct Gemstone.GemFiatSession
import Primitives

public extension GemFiatQuoteServiceProtocol {
    var currency: Primitives.Currency {
        getCurrency().toPrimitives()
    }

    func newSession(type: FiatQuoteType, amount: Int?) -> GemFiatSession {
        newSession(quoteType: type.toGem(), amount: amount.map { UInt32($0) })
    }

    func amountCheck(type: FiatQuoteType, amount: Double, quote: FiatQuote?, available: BigInt) -> GemFiatAmountCheck {
        amountCheck(quoteType: type.toGem(), amount: amount, quote: quote, available: BigUInt(available))
    }

    func quoteUrl(asset: Asset, quoteId: String) async throws -> FiatQuoteUrl {
        try await quoteUrl(assetId: asset.id.identifier, quoteId: quoteId)
    }
}

public extension GemFiatSession {
    var type: FiatQuoteType {
        quoteType.toPrimitives()
    }
}
