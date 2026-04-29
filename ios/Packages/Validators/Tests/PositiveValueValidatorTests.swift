// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Testing
@testable import Validators

struct PositiveValueValidatorTests {
    @Test
    func validatesPositiveValue() throws {
        let validator = PositiveValueValidator<BigInt>()
        try validator.validate(1)
        try validator.validate(123_456)
    }

    @Test
    func throwsOnZero() {
        let validator = PositiveValueValidator<BigInt>()
        #expect(throws: TransferError.invalidAmount) {
            try validator.validate(0)
        }
    }

    @Test
    func throwsOnNegative() {
        let validator = PositiveValueValidator<BigInt>()
        #expect(throws: TransferError.invalidAmount) {
            try validator.validate(-42)
        }
    }

    @Test
    func silentValidation() {
        let validator = PositiveValueValidator<BigInt>().silent

        #expect(throws: SilentValidationError.self) {
            try validator.validate(0)
        }

        #expect(throws: SilentValidationError.self) {
            try validator.validate(-7)
        }
    }
}
