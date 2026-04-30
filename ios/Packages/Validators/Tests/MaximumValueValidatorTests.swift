// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Primitives
import Testing
@testable import Validators

struct MaximumValueValidatorTests {
    private let max = BigInt(100)
    private let maxText = "100 USDC"

    @Test
    func passesEqualOrLess() throws {
        let validator = MaximumValueValidator<BigInt>(
            maximumValue: max,
            maximumValueText: maxText,
        )
        try validator.validate(max - 1)
        try validator.validate(max)
    }

    @Test
    func throwsAboveMaximum() {
        let validator = MaximumValueValidator<BigInt>(
            maximumValue: max,
            maximumValueText: maxText,
        )
        #expect(throws: AnyError("Maximum allowed value is \(maxText)")) {
            try validator.validate(max + 1)
        }
    }
}
