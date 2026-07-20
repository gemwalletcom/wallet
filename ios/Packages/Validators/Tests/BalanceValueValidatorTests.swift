// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Primitives
import Testing
@testable import Validators

struct BalanceValueValidatorTests {
    private let asset = Asset.mock()
    private let available = BigInt(50)

    @Test
    func passesWithinBalance() throws {
        let validator = BalanceValueValidator(
            available: available,
            asset: asset,
        )
        try validator.validate(available)
        try validator.validate(available - 10)
    }

    @Test
    func throwsExceedingBalance() {
        let validator = BalanceValueValidator(
            available: available,
            asset: asset,
        )
        #expect(throws: TransferAmountCalculatorError.insufficientBalance(
            asset,
            requirement: BalanceRequirement(required: available + 1, available: available),
        )) {
            try validator.validate(available + 1)
        }
    }
}
