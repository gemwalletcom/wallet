// Copyright (c). Gem Wallet. All rights reserved.

@testable import FiatConnect
import Formatters
import Primitives
import PrimitivesTestKit
import Testing

struct FiatQuoteViewModelTests {
    let usFormatter = CurrencyFormatter(locale: .US, currencyCode: Currency.usd.rawValue)
    let ukFormatter = CurrencyFormatter(locale: .UK, currencyCode: Currency.usd.rawValue)
    let uaFormatter = CurrencyFormatter(locale: .UA, currencyCode: Currency.usd.rawValue)
    let frFormatter = CurrencyFormatter(locale: .FR, currencyCode: Currency.usd.rawValue)

    @Test
    func buyAmount() {
        #expect(FiatQuoteViewModel(asset: .mock(), quote: .mock(), formatter: usFormatter).amountText == "0.00 BTC")
        #expect(FiatQuoteViewModel(asset: .mock(), quote: .mock(fiatAmount: 10.123, cryptoAmount: 15.12), formatter: usFormatter).amountText == "15.12 BTC")
        #expect(FiatQuoteViewModel(asset: .mock(), quote: .mock(fiatAmount: 10, cryptoAmount: 15), formatter: usFormatter).amountText == "15.00 BTC")
    }

    @Test
    func sellAmount() {
        #expect(FiatQuoteViewModel(asset: .mock(), quote: .mock(type: .sell), formatter: usFormatter).amountText == "0.00 BTC")
        #expect(FiatQuoteViewModel(asset: .mock(), quote: .mock(fiatAmount: 10.123, cryptoAmount: 15.12, type: .sell), formatter: usFormatter).amountText == "15.12 BTC")
        #expect(FiatQuoteViewModel(asset: .mock(), quote: .mock(fiatAmount: 10, cryptoAmount: 15, type: .sell), formatter: usFormatter).amountText == "15.00 BTC")
    }

    @Test
    func buyFiatTextIsAssetPriceTimesCryptoAmount() {
        let quote = FiatQuote.mock(fiatAmount: 50, cryptoAmount: 0.000488)
        let model = FiatQuoteViewModel(asset: .mock(), quote: quote, assetPrice: 100_000, formatter: usFormatter)

        #expect(model.subtitleExtra == "$48.80")
    }

    @Test
    func buyFiatTextFallsBackToRawAmountWhenAssetPriceMissing() {
        let quote = FiatQuote.mock(fiatAmount: 50, cryptoAmount: 0.000488)

        #expect(FiatQuoteViewModel(asset: .mock(), quote: quote, assetPrice: nil, formatter: usFormatter).subtitleExtra == "$50.00")
        #expect(FiatQuoteViewModel(asset: .mock(), quote: quote, assetPrice: 0, formatter: usFormatter).subtitleExtra == "$50.00")
    }

    @Test
    func sellFiatTextUsesRawQuoteFiatAmount() {
        let quote = FiatQuote.mock(fiatAmount: 100, cryptoAmount: 0.05117, type: .sell)
        let model = FiatQuoteViewModel(asset: .mock(), quote: quote, assetPrice: 100_000, formatter: usFormatter)

        #expect(model.subtitleExtra == "$100.00")
    }

    @Test
    func testRateText() {
        #expect(FiatQuoteViewModel(asset: .mock(), quote: .mock(fiatAmount: 0, cryptoAmount: 0), formatter: usFormatter).rateText == "$NaN")
        #expect(FiatQuoteViewModel(asset: .mock(), quote: .mock(fiatAmount: 10.123, cryptoAmount: 15.12), formatter: usFormatter).rateText == "$0.6695")
        #expect(FiatQuoteViewModel(asset: .mock(), quote: .mock(fiatAmount: 50, cryptoAmount: 0.0018), formatter: usFormatter).rateText == "$27,777.78")

        #expect(FiatQuoteViewModel(asset: .mock(), quote: .mock(fiatAmount: 10.123, cryptoAmount: 15.12), formatter: ukFormatter).rateText == "US$0.6695")
        #expect(FiatQuoteViewModel(asset: .mock(), quote: .mock(fiatAmount: 50, cryptoAmount: 0.0018), formatter: ukFormatter).rateText == "US$27,777.78")

        #expect(FiatQuoteViewModel(asset: .mock(), quote: .mock(fiatAmount: 10.123, cryptoAmount: 15.12), formatter: uaFormatter).rateText == "0,6695 $")
        #expect(FiatQuoteViewModel(asset: .mock(), quote: .mock(fiatAmount: 50, cryptoAmount: 0.0018), formatter: uaFormatter).rateText == "27 777,78 $")

        #expect(FiatQuoteViewModel(asset: .mock(), quote: .mock(fiatAmount: 10.123, cryptoAmount: 15.12), formatter: frFormatter).rateText == "0,6695 $ US")
        #expect(FiatQuoteViewModel(asset: .mock(), quote: .mock(fiatAmount: 50, cryptoAmount: 0.0018), formatter: frFormatter).rateText == "27 777,78 $ US")

        #expect(FiatQuoteViewModel(asset: .mock(), quote: .mock(fiatAmount: 0.000000123456, cryptoAmount: 1), formatter: frFormatter).rateText == "0,0000001235 $ US")
    }
}
