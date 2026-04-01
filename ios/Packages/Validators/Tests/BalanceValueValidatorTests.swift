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
        let validator = BalanceValueValidator<BigInt>(
            available: available,
            asset: asset,
        )
        try validator.validate(available)
        try validator.validate(available - 10)
    }

    @Test
    func throwsExceedingBalance() {
        let validator = BalanceValueValidator<BigInt>(
            available: available,
            asset: asset,
        )
        #expect(throws: TransferAmountCalculatorError.insufficientBalance(asset)) {
            try validator.validate(available + 1)
        }
    }
}
