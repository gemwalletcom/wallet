// Copyright (c). Gem Wallet. All rights reserved.

@testable import Perpetuals
import Primitives
import Testing

struct AutocloseTextValidatorTests {
    @Test
    func validatesTheSameNumberThatWouldBeSubmitted() throws {
        let validator = AutocloseTextValidator(type: .takeProfit, direction: .long, marketPrice: 100)

        try validator.validate("1 234.5")
    }

    @Test
    func acceptsATriggerAboveTheMarketPriceForALongTakeProfit() throws {
        let validator = AutocloseTextValidator(type: .takeProfit, direction: .long, marketPrice: 100)

        try validator.validate("150")
    }
}
