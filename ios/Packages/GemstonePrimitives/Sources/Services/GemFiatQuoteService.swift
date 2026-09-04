// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import enum Gemstone.GemFiatAmountCheck
import protocol Gemstone.GemFiatQuoteServiceProtocol
import Primitives

public extension GemFiatQuoteServiceProtocol {
    var currency: Primitives.Currency {
        Primitives.Currency(core: getCurrency())
    }

    func amountCheck(type: FiatQuoteType, amount: Double, quote: FiatQuote?, available: BigInt) -> GemFiatAmountCheck {
        amountCheck(quoteType: type.map(), amount: amount, quote: quote?.json(), available: BigUInt(available))
    }

    func quotes(type: FiatQuoteType, asset: Asset, amount: Double) async throws -> [FiatQuote] {
        try await quotes(quoteType: type.map(), assetId: asset.id.identifier, amount: amount).map { try FiatQuote($0) }
    }

    func quoteUrl(asset: Asset, quoteId: String) async throws -> FiatQuoteUrl {
        try FiatQuoteUrl(await quoteUrl(assetId: asset.id.identifier, quoteId: quoteId))
    }
}
