// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
@testable import Primitives
import Testing

final class CurrencyFormatterTests {
    let currencyFormatterUS = CurrencyFormatter(locale: .US, currencyCode: Currency.usd.rawValue)
    let currencyFormatterUK = CurrencyFormatter(locale: .UK, currencyCode: Currency.gbp.rawValue)

    let abbreviatedFormatterUS = CurrencyFormatter(type: .abbreviated, locale: .US, currencyCode: Currency.usd.rawValue)
    let abbreviatedFormatterUK = CurrencyFormatter(type: .abbreviated, locale: .UK, currencyCode: Currency.gbp.rawValue)

    @Test
    func currency() {
        #expect(currencyFormatterUS.string(0) == "$0.00")
        #expect(currencyFormatterUS.string(11.12) == "$11.12")
        #expect(currencyFormatterUS.string(11) == "$11.00")
        #expect(currencyFormatterUS.string(-1.2) == "-$1.20")
        #expect(currencyFormatterUS.string(12_000_123) == "$12,000,123.00")
        #expect(currencyFormatterUS.string(0.0000000002) == "$0.0000000002")
        #expect(currencyFormatterUS.string(0.0000000000001) == "$0.00")
    }

    @Test
    func smallValue() {
        #expect(currencyFormatterUS.string(0.10) == "$0.1")
        #expect(currencyFormatterUS.string(0.11) == "$0.11")
        #expect(currencyFormatterUS.string(0.2) == "$0.2")
        #expect(currencyFormatterUS.string(0.99) == "$0.99")
        #expect(currencyFormatterUS.string(1.89999) == "$1.90")
        #expect(currencyFormatterUS.string(0.70) == "$0.7")
        #expect(currencyFormatterUS.string(0.0345) == "$0.0345")
        #expect(currencyFormatterUS.string(0.01) == "$0.01")
        #expect(currencyFormatterUS.string(0.13) == "$0.13")
        #expect(currencyFormatterUS.string(0.0123) == "$0.0123")
        #expect(currencyFormatterUS.string(0.002) == "$0.002")
        #expect(currencyFormatterUS.string(0.001) == "$0.001")
        #expect(currencyFormatterUS.string(0.000123456) == "$0.0001235")
        #expect(currencyFormatterUS.string(0.00000123) == "$0.00000123")
    }

    @Test
    func currencyGBPLocale() {
        #expect(currencyFormatterUK.string(0.0002) == "£0.0002")
        #expect(currencyFormatterUK.string(11.12) == "£11.12")
        #expect(currencyFormatterUK.string(11) == "£11.00")
        #expect(currencyFormatterUK.string(12_000_123) == "£12,000,123.00")
    }

    @Test
    func testAbbreviated() {
        #expect(abbreviatedFormatterUS.string(0) == "$0.00")
        #expect(abbreviatedFormatterUS.string(12) == "$12.00")
        #expect(abbreviatedFormatterUS.string(1234) == "$1,234.00")
        #expect(abbreviatedFormatterUS.string(10000) == "$10,000.00")
        #expect(abbreviatedFormatterUS.string(-1234) == "-$1,234.00")
        #expect(abbreviatedFormatterUS.string(-10000) == "-$10,000.00")
        #expect(abbreviatedFormatterUS.string(-5_600_000) == "-$5.6M")

        #expect(abbreviatedFormatterUK.string(123_456) == "£120k")
        #expect(abbreviatedFormatterUK.string(5_000_000) == "£5m")
        #expect(abbreviatedFormatterUK.string(7_890_000_000) == "£7.9bn")
        #expect(abbreviatedFormatterUK.string(1_200_000_000_000) == "£1.2tn")
        #expect(abbreviatedFormatterUK.string(-9_999_999_999) == "-£10bn")
    }
}
