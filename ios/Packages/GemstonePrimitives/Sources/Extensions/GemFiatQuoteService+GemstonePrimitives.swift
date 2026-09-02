// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import enum Gemstone.GemFiatAmountCheck
import protocol Gemstone.GemFiatQuoteServiceProtocol
import Primitives

public extension GemFiatQuoteServiceProtocol {
    var currencyCode: String {
        Currency(core: currency()).rawValue
    }

    func amountCheck(type: FiatQuoteType, amount: Double, quote: FiatQuote?, available: BigInt) -> GemFiatAmountCheck {
        amountCheck(quoteType: type.json(), amount: amount, quote: quote?.json(), available: available.description)
    }

    func quotes(walletId: WalletId, type: FiatQuoteType, asset: Asset, amount: Double) async throws -> [FiatQuote] {
        try await quotes(walletId: walletId.id, quoteType: type.json(), assetId: asset.id.identifier, amount: amount).map { try FiatQuote($0) }
    }

    func quoteUrl(walletId: WalletId, asset: Asset, quoteId: String) async throws -> FiatQuoteUrl {
        try FiatQuoteUrl(await quoteUrl(walletId: walletId.id, assetId: asset.id.identifier, quoteId: quoteId))
    }
}
