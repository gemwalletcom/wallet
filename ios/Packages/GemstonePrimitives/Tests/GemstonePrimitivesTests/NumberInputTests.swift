// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import GemstonePrimitives
import Primitives
import Testing

struct NumberInputTests {
    @Test
    func valueScalesByDecimals() throws {
        #expect(try NumberInput.value("1,234.56", decimals: 8, locale: .US) == BigInt("123456000000"))
        #expect(try NumberInput.value("1.234,56", decimals: 8, locale: .DA_DK) == BigInt("123456000000"))
    }

    @Test
    func doubleReadsThePlainNumber() {
        #expect(NumberInput.double("1,234.56", locale: .US) == 1234.56)
        #expect(NumberInput.double("1.234,56", locale: .DA_DK) == 1234.56)
    }

    @Test
    func textWithoutANumberIsNotAnAmount() {
        #expect(NumberInput.double("abc", locale: .US) == nil)
        #expect(throws: (any Error).self) {
            try NumberInput.value("abc", decimals: 8, locale: .US)
        }
    }
}
