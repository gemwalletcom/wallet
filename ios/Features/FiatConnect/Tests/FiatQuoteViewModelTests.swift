// Copyright (c). Gem Wallet. All rights reserved.

@testable import FiatConnect
import Foundation
import Formatters
import GemstonePrimitives
import struct Gemstone.GemAssetRate
import func Gemstone.formattedCurrency
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
import Testing

struct FiatQuoteViewModelTests {

    @Test
    func amountText() {
        #expect(model(cryptoAmount: 0).amountText == "0 BTC")
        #expect(model(cryptoAmount: 15.12).amountText == "15.12 BTC")
        #expect(model(cryptoAmount: 15).amountText == "15 BTC")
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
    func rateTextNamesTheAssetAndFollowsTheLocale() {
        #expect(model(rate: 0.669510582, locale: .US).rateText == "1 BTC ≈ $0.6695")
        #expect(model(rate: 27_777.7777778, locale: .US).rateText == "1 BTC ≈ $27,777.78")

        #expect(model(rate: 0.669510582, locale: .UK).rateText == "1 BTC ≈ US$0.6695")
        #expect(model(rate: 27_777.7777778, locale: .UK).rateText == "1 BTC ≈ US$27,777.78")

        #expect(model(rate: 0.669510582, locale: .UA).rateText == "1 BTC ≈ 0,6695 $")
        #expect(model(rate: 27_777.7777778, locale: .UA).rateText == "1 BTC ≈ 27 777,78 $")

        #expect(model(rate: 0.669510582, locale: .FR).rateText == "1 BTC ≈ 0,6695 $ US")
        #expect(model(rate: 27_777.7777778, locale: .FR).rateText == "1 BTC ≈ 27 777,78 $ US")

        #expect(model(rate: 0.000000123456, locale: .FR).rateText == "1 BTC ≈ 0,0000001235 $ US")
    }

    private func model(
        cryptoAmount: Double = 0,
        fiatAmount: Double = 0,
        rate: Double? = nil,
        locale: Locale = .US,
    ) -> FiatQuoteViewModel {
        let asset = Asset.mock()
        return FiatQuoteViewModel(
            asset: asset,
            row: .mock(
                cryptoAmount: cryptoAmount,
                fiatAmount: fiatAmount,
                rate: rate.map {
                    GemAssetRate(
                        baseSymbol: asset.symbol,
                        quoteSymbol: Currency.usd.rawValue,
                        value: formattedCurrency(value: $0, code: Currency.usd.rawValue, style: .currency),
                    )
                },
            ),
            locale: locale,
        )
    }
}
