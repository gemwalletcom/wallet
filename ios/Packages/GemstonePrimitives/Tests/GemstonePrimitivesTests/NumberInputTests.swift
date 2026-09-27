// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Primitives
import Testing

struct NumberInputTests {
    @Test
    func doubleReadsThePlainNumber() {
        #expect(NumberInput.double("1,234.56", locale: .US) == 1234.56)
        #expect(NumberInput.double("1.234,56", locale: .DA_DK) == 1234.56)
    }

    @Test
    func textWithoutANumberIsNotAnAmount() {
        #expect(NumberInput.double("abc", locale: .US) == nil)
    }
}
