// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
@testable import GemstonePrimitives
import Primitives
import Testing

final class GemPaymentQuoteTests {
    private let expiresAtSeconds: Int64 = 1_700_000_000

    private func gemQuote(assetId: String = "ethereum_0xtoken") -> GemPaymentQuote {
        GemPaymentQuote(
            id: "option_1",
            paymentId: "pay_1",
            amount: GemPaymentAmount(assetId: assetId, value: "10", symbol: "USDT", decimals: 6),
            expiresAt: expiresAtSeconds,
            collectDataUrl: .none,
            providerData: #"{"opaque":true}"#,
        )
    }

    @Test
    func gatewaySecondsBecomeADate() throws {
        let quote = try gemQuote().map()

        #expect(quote.expiresAt == Date(timeIntervalSince1970: TimeInterval(expiresAtSeconds)))
        #expect(quote.amount.assetId.chain == .ethereum)
        #expect(quote.amount.assetId.tokenId == "0xtoken")
    }

    @Test
    func quoteReturnsToTheGatewayUnchanged() throws {
        #expect(try gemQuote().map().map() == gemQuote())
        #expect(try gemQuote(assetId: "ethereum").map().map() == gemQuote(assetId: "ethereum"))
    }
}
