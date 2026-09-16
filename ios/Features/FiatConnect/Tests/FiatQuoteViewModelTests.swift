// Copyright (c). Gem Wallet. All rights reserved.

@testable import FiatConnect
@testable import FiatConnectTestKit
import Foundation
import Formatters
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
        #expect(FiatQuoteViewModel.mock(row: .mock(fiatAmount: 48.8)).subtitleExtra == "$48.80")
        #expect(FiatQuoteViewModel.mock(row: .mock(fiatAmount: 100)).subtitleExtra == "$100.00")
    }

    @Test
    func rateTextIsEmptyWithoutARate() {
        #expect(FiatQuoteViewModel.mock(row: .mock(rate: nil)).rateText == "")
    }

    @Test
    func rateTextNamesTheAssetAndFollowsTheLocale() {
        #expect(FiatQuoteViewModel.mock(row: .mock(rate: 0.669510582)).rateText == "1 BTC ≈ $0.6695")
        #expect(FiatQuoteViewModel.mock(row: .mock(rate: 27_777.7777778)).rateText == "1 BTC ≈ $27,777.78")

        #expect(FiatQuoteViewModel.mock(row: .mock(rate: 0.669510582), locale: .UK).rateText == "1 BTC ≈ US$0.6695")
        #expect(FiatQuoteViewModel.mock(row: .mock(rate: 27_777.7777778), locale: .UK).rateText == "1 BTC ≈ US$27,777.78")

        #expect(FiatQuoteViewModel.mock(row: .mock(rate: 0.669510582), locale: .UA).rateText == "1 BTC ≈ 0,6695 $")
        #expect(FiatQuoteViewModel.mock(row: .mock(rate: 27_777.7777778), locale: .UA).rateText == "1 BTC ≈ 27 777,78 $")

        #expect(FiatQuoteViewModel.mock(row: .mock(rate: 0.669510582), locale: .FR).rateText == "1 BTC ≈ 0,6695 $ US")
        #expect(FiatQuoteViewModel.mock(row: .mock(rate: 27_777.7777778), locale: .FR).rateText == "1 BTC ≈ 27 777,78 $ US")

        #expect(FiatQuoteViewModel.mock(row: .mock(rate: 0.000000123456), locale: .FR).rateText == "1 BTC ≈ 0,0000001235 $ US")
    }
}
