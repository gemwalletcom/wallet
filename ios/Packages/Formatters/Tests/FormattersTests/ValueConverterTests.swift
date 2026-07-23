// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
@testable import Formatters
import Foundation
import Primitives
import PrimitivesTestKit
import Testing

struct ValueConverterTests {
    let converter = ValueConverter()

    @Test
    func testConvertToFiat() throws {
        let price = AssetPrice.mock(price: 2.5)
        #expect(try converter.convertToFiat(amount: "1", price: price) == 2.5)
        #expect(try converter.convertToFiat(amount: "0.4", price: price) == 1.0)
        #expect(try converter.convertToFiat(amount: "10", price: price) == 25.0)
        #expect(try converter.convertToFiat(amount: "0", price: price) == 0.0)
    }

    @Test
    func convertToAmount() throws {
        let price = AssetPrice.mock(price: 2.5)
        #expect(try converter.convertToDisplayedAmount(fiatValue: "2.5", price: price, decimals: 8) == BigInt(100_000_000))
        #expect(try converter.convertToDisplayedAmount(fiatValue: "1.0", price: price, decimals: 8) == BigInt(40_000_000))
        #expect(try converter.convertToDisplayedAmount(fiatValue: "25", price: price, decimals: 8) == BigInt(1_000_000_000))
    }

    @Test
    func convertToFiatWithZeroAmount() throws {
        #expect(try converter.convertToFiat(amount: "0", price: .mock(price: 2.5)) == 0.0)
    }

    @Test
    func convertToAmountWithInvalidPrice() {
        #expect(throws: (any Error).self) {
            try converter.convertToDisplayedAmount(fiatValue: "10", price: .mock(price: 0), decimals: 8)
        }
        #expect(throws: (any Error).self) {
            try converter.convertToDisplayedAmount(fiatValue: "10", price: .mock(price: -1), decimals: 8)
        }
    }

    @Test
    func convertToAmountWithZeroFiatValue() throws {
        #expect(throws: AnyError.self) {
            try converter.convertToDisplayedAmount(fiatValue: "0", price: .mock(price: 2.5), decimals: 8)
        }
    }

    @Test
    func convertToFiatWithSmallAmount() throws {
        #expect(try converter.convertToFiat(amount: "0.00000001", price: .mock(price: 2.5)) == 0.000000025)
    }

    @Test
    func convertToAmountWithSmallFiatValue() throws {
        #expect(try converter.convertToDisplayedAmount(fiatValue: "0.000000025", price: .mock(price: 2.5), decimals: 8) == BigInt(1))
    }

    @Test
    func convertToAmountWithRounding() throws {
        let price = AssetPrice.mock(price: 3.33333333)
        #expect(try converter.convertToDisplayedAmount(fiatValue: "10", price: price, decimals: 2) == BigInt(300))
    }

    @Test(arguments: ["de_DE", "it_IT", "da_DK", "en_US"])
    func convertToDisplayedAmountWithThreeDecimals(identifier: String) throws {
        let converter = ValueConverter(formatter: ValueFormatter(locale: Locale(identifier: identifier), style: .auto))
        #expect(try converter.convertToDisplayedAmount(fiatValue: "1234", price: .mock(price: 1000.0), decimals: 6) == BigInt(1_230_000))
    }

    @Test(arguments: ["de_DE", "es_ES", "it_IT", "nl_NL", "pt_BR", "en_US", "fr_FR", "ja_JP"])
    func convertToAmountAboveGroupingThreshold(identifier: String) throws {
        let converter = ValueConverter(formatter: ValueFormatter(locale: Locale(identifier: identifier), style: .auto))
        let price = AssetPrice.mock(price: 1.0)
        let unit = BigInt(10).power(6)

        #expect(try converter.convertToDisplayedAmount(fiatValue: "1000", price: price, decimals: 6) == BigInt(1000) * unit)
        #expect(try converter.convertToDisplayedAmount(fiatValue: "1000000", price: price, decimals: 6) == BigInt(1_000_000) * unit)
        #expect(try converter.convertToDisplayedAmount(fiatValue: "12345678", price: price, decimals: 6) == BigInt(12_345_678) * unit)
    }

    @Test(arguments: ["de_DE", "it_IT", "en_US", "ja_JP"])
    func displayedNumberIsIndependentOfFormatterLocale(identifier: String) throws {
        let formatter = ValueFormatter(locale: Locale(identifier: identifier), style: .auto)
        #expect(try formatter.displayedNumber(from: Decimal(1_000_000), decimals: 6) == BigInt(1_000_000) * BigInt(10).power(6))
        #expect(try formatter.displayedNumber(from: #require(Decimal(string: "1.234")), decimals: 6) == BigInt(1_230_000))
    }
}
