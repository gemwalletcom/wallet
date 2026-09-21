// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import struct Gemstone.FiatProvider
import enum Gemstone.FiatProviderName
import struct Gemstone.FiatQuote
import enum Gemstone.FiatQuoteType
import func Gemstone.formattedAmount
import func Gemstone.formattedCurrency
import struct Gemstone.GemAssetRate
import struct Gemstone.GemFiatQuoteRequest
import struct Gemstone.GemFiatQuoteRow
import struct Gemstone.GemFiatQuotesResult
import enum Gemstone.GemServiceError
import GemstonePrimitives
import Primitives
import PrimitivesTestKit

public extension FiatQuote {
    static func mock(
        id: String = UUID().uuidString,
        fiatAmount: Double = 0,
        cryptoAmount: Double = 0,
        type: FiatQuoteType = .buy,
        fiatCurrency: String = Primitives.Currency.usd.rawValue,
        providerId: FiatProviderName = .moonPay,
    ) -> FiatQuote {
        FiatQuote(
            id: id,
            asset: Primitives.Asset.mock().toGem(),
            provider: .mock(id: providerId),
            quoteType: type,
            fiatAmount: fiatAmount,
            fiatCurrency: fiatCurrency,
            cryptoAmount: cryptoAmount,
            value: BigUInt(0),
            latency: 0,
            paymentMethods: [],
        )
    }
}

public extension FiatProvider {
    static func mock(id: FiatProviderName = .moonPay, name: String = "") -> FiatProvider {
        FiatProvider(
            id: id,
            name: name,
            imageUrl: "",
            priority: nil,
            thresholdBps: nil,
            enabled: true,
            buyEnabled: true,
            sellEnabled: true,
            paymentMethods: [],
        )
    }
}

public extension GemFiatQuoteRow {
    static func mock(
        quoteId: String = UUID().uuidString,
        provider: FiatProviderName = .moonPay,
        providerName: String = "",
        cryptoAmount: Double = 0,
        fiatAmount: Double = 0,
        rate: Double? = nil,
    ) -> GemFiatQuoteRow {
        GemFiatQuoteRow(
            quoteId: quoteId,
            provider: provider,
            providerName: providerName,
            cryptoAmount: formattedAmount(value: cryptoAmount, symbol: "BTC", style: .auto),
            fiatAmount: formattedCurrency(value: fiatAmount, code: "USD", style: .fiat),
            rate: rate.map { GemAssetRate(baseSymbol: "BTC", value: formattedCurrency(value: $0, code: "USD", style: .currency)) },
        )
    }
}

public extension GemFiatQuotesResult {
    static func mock(
        quotes: [FiatQuote] = [],
        amount: Double = 50,
        type: FiatQuoteType = .buy,
        error: GemServiceError? = nil,
    ) -> GemFiatQuotesResult {
        GemFiatQuotesResult(
            request: GemFiatQuoteRequest(quoteType: type, amount: amount),
            quotes: quotes,
            error: error,
        )
    }
}
