// Copyright (c). Gem Wallet. All rights reserved.

@testable import FiatConnect
import Formatters
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
import Testing

struct FiatQuoteViewModelTests {
    let usFormatter = CurrencyFormatter(locale: .US, currencyCode: Currency.usd.rawValue)
    let ukFormatter = CurrencyFormatter(locale: .UK, currencyCode: Currency.usd.rawValue)
    let uaFormatter = CurrencyFormatter(locale: .UA, currencyCode: Currency.usd.rawValue)
    let frFormatter = CurrencyFormatter(locale: .FR, currencyCode: Currency.usd.rawValue)

    @Test
    func amountText() {
        #expect(model(cryptoAmount: 0).amountText == "0.00 BTC")
        #expect(model(cryptoAmount: 15.12).amountText == "15.12 BTC")
        #expect(model(cryptoAmount: 15).amountText == "15.00 BTC")
    }

    @Test
    func fiatTextIsTheAmountCoreResolved() {
        #expect(model(fiatAmount: 48.8).subtitleExtra == "$48.80")
        #expect(model(fiatAmount: 100).subtitleExtra == "$100.00")
    }

    @Test
    func rateTextIsEmptyWithoutARate() {
        #expect(model(rate: nil).rateText == "")
    }

    @Test
    func rateTextFollowsTheLocale() {
        #expect(model(rate: 0.669510582, formatter: usFormatter).rateText == "$0.6695")
        #expect(model(rate: 27_777.7777778, formatter: usFormatter).rateText == "$27,777.78")

        #expect(model(rate: 0.669510582, formatter: ukFormatter).rateText == "US$0.6695")
        #expect(model(rate: 27_777.7777778, formatter: ukFormatter).rateText == "US$27,777.78")

        #expect(model(rate: 0.669510582, formatter: uaFormatter).rateText == "0,6695 $")
        #expect(model(rate: 27_777.7777778, formatter: uaFormatter).rateText == "27 777,78 $")

        #expect(model(rate: 0.669510582, formatter: frFormatter).rateText == "0,6695 $ US")
        #expect(model(rate: 27_777.7777778, formatter: frFormatter).rateText == "27 777,78 $ US")

        #expect(model(rate: 0.000000123456, formatter: frFormatter).rateText == "0,0000001235 $ US")
    }

    private func model(
        cryptoAmount: Double = 0,
        fiatAmount: Double = 0,
        rate: Double? = nil,
        formatter: CurrencyFormatter? = nil,
    ) -> FiatQuoteViewModel {
        FiatQuoteViewModel(
            asset: .mock(),
            row: .mock(cryptoAmount: cryptoAmount, fiatAmount: fiatAmount, rate: rate),
            formatter: formatter ?? usFormatter,
        )
    }
}
