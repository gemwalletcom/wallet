// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import Primitives
import PrimitivesTestKit
import Testing

final class PriceAlertTests {
    @Test func testId() {
        #expect(PriceAlert.mock().id == "bitcoin")
        #expect(PriceAlert.mock(price: 100, priceDirection: .up).id == "bitcoin_USD_100_up")
        #expect(PriceAlert.mock(price: 1.12344, priceDirection: .down).id == "bitcoin_USD_1.12344_down")
        #expect(PriceAlert.mock(pricePercentChange: 5, priceDirection: .up).id == "bitcoin_USD_5_up")
        #expect(PriceAlert.mock(pricePercentChange: 10000.10, priceDirection: .down).id == "bitcoin_USD_10000.1_down")
        #expect(PriceAlert.mock(price: 1, priceDirection: .up).id == "bitcoin_USD_1_up")
        #expect(PriceAlert.mock(pricePercentChange: 0.23).id == "bitcoin_USD_0.23")
        #expect(PriceAlert.mock(price: 50000.01).id == "bitcoin_USD_50000.01")
        #expect(PriceAlert.mock(price: 0.001234567).id == "bitcoin_USD_0.001234567")
    }

    @Test func autoAlertType() {
        #expect(PriceAlert.mock(assetId: .mock()).type == .auto)
    }

    @Test func priceAlertType() {
        let priceAlert = PriceAlert.mock(
            assetId: .mock(),
            price: 3000,
            priceDirection: .up,
        )
        #expect(priceAlert.type == .price)
    }

    @Test func percentChangeAlertType() {
        let percentChangeAlert = PriceAlert.mock(
            pricePercentChange: 5.0,
            priceDirection: .down,
        )
        #expect(percentChangeAlert.type == .pricePercentChange)
    }

    @Test func priceAndPercentAlertType() {
        let priceAndPercentAlert = PriceAlert.mock(
            price: 1.2,
            pricePercentChange: 3.5,
        )
        #expect(priceAndPercentAlert.type == .auto)
    }
}
