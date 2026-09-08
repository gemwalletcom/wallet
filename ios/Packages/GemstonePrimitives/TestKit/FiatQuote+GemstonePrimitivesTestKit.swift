// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import struct Gemstone.FiatProvider
import enum Gemstone.FiatProviderName
import struct Gemstone.FiatQuote
import enum Gemstone.FiatQuoteType
import struct Gemstone.GemFiatQuoteRow
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
            asset: Primitives.Asset.mock().map(),
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
        providerImageUrl: String? = nil,
        cryptoAmount: Double = 0,
        fiatAmount: Double = 0,
        rate: Double? = nil,
    ) -> GemFiatQuoteRow {
        GemFiatQuoteRow(
            quoteId: quoteId,
            provider: provider,
            providerName: providerName,
            providerImageUrl: providerImageUrl,
            cryptoAmount: cryptoAmount,
            fiatAmount: fiatAmount,
            rate: rate,
        )
    }
}
