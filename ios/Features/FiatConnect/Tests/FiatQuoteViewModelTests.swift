// Copyright (c). Gem Wallet. All rights reserved.

@testable import FiatConnect
@testable import FiatConnectTestKit
import Formatters
import Foundation
import func Gemstone.formattedAmount
import func Gemstone.formattedCurrency
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
import Testing

struct FiatQuoteViewModelTests {
    @Test
    func amountText() {
        #expect(FiatQuoteViewModel.mock(row: .mock(cryptoAmount: formattedAmount(value: 0, symbol: "BTC", style: .auto), fiatAmount: formattedCurrency(value: 0, code: "USD", style: .fiat))).amountText == "0 BTC")
        #expect(FiatQuoteViewModel.mock(row: .mock(cryptoAmount: formattedAmount(value: 15.12, symbol: "BTC", style: .auto), fiatAmount: formattedCurrency(value: 0, code: "USD", style: .fiat))).amountText == "15.12 BTC")
        #expect(FiatQuoteViewModel.mock(row: .mock(cryptoAmount: formattedAmount(value: 15, symbol: "BTC", style: .auto), fiatAmount: formattedCurrency(value: 0, code: "USD", style: .fiat))).amountText == "15 BTC")
    }

    @Test
    func fiatTextIsTheAmountCoreResolved() {
        #expect(FiatQuoteViewModel.mock(row: .mock(cryptoAmount: formattedAmount(value: 0, symbol: "BTC", style: .auto), fiatAmount: formattedCurrency(value: 48.8, code: "USD", style: .fiat))).listItem.subtitleExtra == "$48.80")
        #expect(FiatQuoteViewModel.mock(row: .mock(cryptoAmount: formattedAmount(value: 0, symbol: "BTC", style: .auto), fiatAmount: formattedCurrency(value: 100, code: "USD", style: .fiat))).listItem.subtitleExtra == "$100.00")
    }
}
