// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Primitives
import Testing

@testable import Validators

struct MinimumValueValidatorTests {
    private let min = BigInt(10)
    private let asset = Asset.mockBNB()

    @Test
    func passesEqualOrGreater() throws {
        let validator = MinimumValueValidator<BigInt>(
            minimumValue: min,
            asset: asset,
        )
        try validator.validate(min)
        try validator.validate(min + 1)
    }

    @Test
    func throwsBelowMinimum() {
        let validator = MinimumValueValidator<BigInt>(
            minimumValue: min,
            asset: asset,
        )
        #expect(throws: TransferError.minimumAmount(asset: asset, required: min)) {
            try validator.validate(min - 1)
        }
    }
}
