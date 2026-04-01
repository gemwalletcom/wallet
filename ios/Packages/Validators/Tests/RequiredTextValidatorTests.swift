// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Testing

@testable import Validators

struct RequiredTextValidatorTests {
    private let fieldName = "Wallet 1"

    @Test
    func validatesNonEmpty() throws {
        let validator = RequiredTextValidator(requireName: fieldName)
        try validator.validate("Alice")
    }

    @Test
    func throwsOnEmpty() {
        let validator = RequiredTextValidator(requireName: fieldName)
        #expect(throws: RequiredFieldError(field: fieldName)) {
            try validator.validate("")
        }
    }

    @Test
    func throwsOnWhitespaceOnly() {
        let validator = RequiredTextValidator(requireName: fieldName)
        #expect(throws: RequiredFieldError(field: fieldName)) {
            try validator.validate(" \n\t")
        }
    }
}
