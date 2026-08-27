// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import Primitives
import PrimitivesTestKit
import Testing

final class PriceAlertTests {
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
