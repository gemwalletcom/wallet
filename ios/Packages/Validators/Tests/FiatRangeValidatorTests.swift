// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization
import Primitives
import Testing
@testable import Validators

struct FiatRangeValidatorTests {
    private let range = 10.0 ... 10000.0
    private let minimumText = "$10"
    private let maximumText = "$10,000"

    var validator: FiatRangeValidator<Double> {
        FiatRangeValidator<Double>(
            range: range,
            minimumValueText: minimumText,
            maximumValueText: maximumText,
        )
    }

    @Test
    func passesWithinRange() throws {
        try validator.validate(10)
        try validator.validate(10000)
    }

    @Test
    func throwsBelowMinimum() {
        #expect(throws: AnyError(Localized.Transfer.minimumAmount(minimumText))) {
            try validator.validate(9)
        }
    }

    @Test
    func throwsAboveMaximum() {
        #expect(throws: AnyError(Localized.Transfer.maximumAmount(maximumText))) {
            try validator.validate(10001)
        }
    }
}
