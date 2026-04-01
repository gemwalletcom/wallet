// Copyright (c). Gem Wallet. All rights reserved.

@testable import Formatters
import Foundation
import Testing

struct LatinNumericConverterTests {
    @Test
    func arabicIndicDigits() throws {
        #expect(LatinNumericConverter.toLatinDigits("٤٥٦") == "456")
    }

    @Test
    func extendedArabicDigits() throws {
        #expect(LatinNumericConverter.toLatinDigits("۹۸۷") == "987")
    }

    @Test
    func arabicDecimalSeparator() throws {
        #expect(LatinNumericConverter.toLatinDigits("١٢٣٫٤٥") == "123.45")
    }

    @Test
    func arabicThousandsSeparator() throws {
        #expect(LatinNumericConverter.toLatinDigits("١٢٬٣٤٥") == "12345")
    }

    @Test
    func arabicFullExample() throws {
        let input = "١٢٬٣٤٥٬٦٧٨٫٩٠١٢٣"
        let expected = "12345678.90123"
        #expect(LatinNumericConverter.toLatinDigits(input) == expected)
    }

    @Test
    func arabicDigitsWithSuffix() throws {
        #expect(LatinNumericConverter.toLatinDigits("٤٥٦BTC") == "456BTC")
    }

    @Test
    func latinDigitsWithArabicDecimal() throws {
        #expect(LatinNumericConverter.toLatinDigits("123٫45") == "123.45")
    }

    @Test
    func emptyString() throws {
        #expect(LatinNumericConverter.toLatinDigits("") == "")
    }
}
