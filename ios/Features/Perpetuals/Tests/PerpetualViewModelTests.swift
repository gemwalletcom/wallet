// Copyright (c). Gem Wallet. All rights reserved.

@testable import Perpetuals
import GemstonePrimitives
import PerpetualsTestKit
import Primitives
import PrimitivesTestKit
import Testing

struct PerpetualViewModelTests {
    @Test
    func name() {
        #expect(PerpetualViewModel(perpetual: .mock(name: "BTC-PERP"), asset: .mock()).name == "BTC-PERP")
    }

    @Test
    func volumeField() {
        #expect(PerpetualViewModel(perpetual: .mock(volume24h: 1_500_000), asset: .mock()).row.volume24h.text() == "$1.5M")
    }

    @Test
    func openInterestField() {
        #expect(PerpetualViewModel(perpetual: .mock(openInterest: 5_250_000), asset: .mock()).row.openInterest.text() == "$5.25M")
    }

    @Test
    func fundingRateField() {
        #expect(PerpetualViewModel(perpetual: .mock(funding: 0.0013), asset: .mock()).row.fundingApr.text() == "+11.39%")
    }

    @Test
    func priceText() {
        #expect(PerpetualViewModel(perpetual: .mock(price: 45000), asset: .mock()).priceText == "$45,000.00")
        #expect(PerpetualViewModel(perpetual: .mock(price: 0.5), asset: .mock()).priceText == "$0.5")
        #expect(PerpetualViewModel(perpetual: .mock(price: 1234.56), asset: .mock()).priceText == "$1,234.56")
    }
}
