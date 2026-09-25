// Copyright (c). Gem Wallet. All rights reserved.

@testable import FiatConnect
@testable import FiatConnectTestKit
import Formatters
import Foundation
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
import Testing

struct FiatQuoteViewModelTests {
    @Test
    func amountText() {
        #expect(FiatQuoteViewModel.mock(row: .mock(cryptoAmount: 0)).amountText == "0 BTC")
        #expect(FiatQuoteViewModel.mock(row: .mock(cryptoAmount: 15.12)).amountText == "15.12 BTC")
        #expect(FiatQuoteViewModel.mock(row: .mock(cryptoAmount: 15)).amountText == "15 BTC")
    }

    @Test
    func fiatTextIsTheAmountCoreResolved() {
        #expect(FiatQuoteViewModel.mock(row: .mock(fiatAmount: 48.8)).listItem.subtitleExtra == "$48.80")
        #expect(FiatQuoteViewModel.mock(row: .mock(fiatAmount: 100)).listItem.subtitleExtra == "$100.00")
    }
}
