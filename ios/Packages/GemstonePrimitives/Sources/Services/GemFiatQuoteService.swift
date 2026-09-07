// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import enum Gemstone.GemFiatAmountCheck
import struct Gemstone.GemFiatSession
import protocol Gemstone.GemFiatQuoteServiceProtocol
import Primitives

public extension GemFiatQuoteServiceProtocol {
    var currency: Primitives.Currency {
        Primitives.Currency(core: getCurrency())
    }

    func newSession(type: FiatQuoteType, amount: Int?) -> GemFiatSession {
        newSession(quoteType: type.map(), amount: amount.map { UInt32($0) })
    }

    func amountCheck(type: FiatQuoteType, amount: Double, quote: FiatQuote?, available: BigInt) -> GemFiatAmountCheck {
        amountCheck(quoteType: type.map(), amount: amount, quote: quote?.json(), available: BigUInt(available))
    }

    func quoteUrl(asset: Asset, quoteId: String) async throws -> FiatQuoteUrl {
        try FiatQuoteUrl(await quoteUrl(assetId: asset.id.identifier, quoteId: quoteId))
    }
}

public extension GemFiatSession {
    var type: FiatQuoteType {
        quoteType.map()
    }

    var amount: String {
        current().amount
    }

    var selectedFiatQuote: FiatQuote? {
        selectedQuote().flatMap { try? FiatQuote($0) }
    }

    var fiatQuotes: [FiatQuote] {
        current().quotes.compactMap { try? FiatQuote($0) }
    }
}
