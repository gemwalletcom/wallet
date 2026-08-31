// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Primitives
import Testing
@testable import Validators

struct AutocloseValidatorTests {
    @Test
    func validatesTheSameNumberThatWouldBeSubmitted() {
        let text = "1,234.5"
        let validator = AutocloseValidator(type: .takeProfit, direction: .long, marketPrice: 100)

        #expect(throws: (any Error).self) {
            try validator.validate(text)
        }
        #expect(NumericFormatter(locale: .current).double(from: text) == 1)
    }

    @Test
    func acceptsATriggerAboveTheMarketPriceForALongTakeProfit() throws {
        let validator = AutocloseValidator(type: .takeProfit, direction: .long, marketPrice: 100)

        try validator.validate("150")
    }
}
