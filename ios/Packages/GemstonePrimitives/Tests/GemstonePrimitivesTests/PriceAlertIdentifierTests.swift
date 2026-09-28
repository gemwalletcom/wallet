// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Testing

struct PriceAlertIdentifierTests {
    @Test func id() {
        #expect(PriceAlert.mock(currency: .usd).id == "bitcoin")
        #expect(PriceAlert.mock(currency: .usd, price: 100, priceDirection: .up).id == "bitcoin_USD_100_up")
        #expect(PriceAlert.mock(currency: .usd, price: 1.12344, priceDirection: .down).id == "bitcoin_USD_1.12344_down")
        #expect(PriceAlert.mock(currency: .usd, pricePercentChange: 5, priceDirection: .up).id == "bitcoin_USD_5_up")
        #expect(PriceAlert.mock(currency: .usd, pricePercentChange: 10000.10, priceDirection: .down).id == "bitcoin_USD_10000.1_down")
        #expect(PriceAlert.mock(currency: .usd, price: 1, priceDirection: .up).id == "bitcoin_USD_1_up")
        #expect(PriceAlert.mock(currency: .usd, pricePercentChange: 0.23).id == "bitcoin_USD_0.23")
        #expect(PriceAlert.mock(currency: .usd, price: 50000.01).id == "bitcoin_USD_50000.01")
        #expect(PriceAlert.mock(currency: .usd, price: 0.001234567).id == "bitcoin_USD_0.001234567")
    }
}
