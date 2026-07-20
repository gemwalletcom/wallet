// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Primitives
import PrimitivesTestKit
import Testing
@testable import Validators

struct MinimumAccountReserveValidatorTests {
    private let nativeAsset = Asset.mockBNB()
    private let nonNativeAsset = Asset.mockEthereumUSDT()
    private let requiredReserve = BigInt(2)
    private let availableAmount = BigInt(10)

    @Test
    func passesWhenRemainingZeroOrAboveReservePlusOne() throws {
        let validator = MinimumAccountReserveValidator(
            available: availableAmount,
            reserve: requiredReserve,
            asset: nativeAsset,
        )

        try validator.validate(availableAmount)
        try validator.validate(availableAmount - requiredReserve - 1)
        try validator.validate(0)
    }

    @Test
    func throwsWhenRemainingBetweenOneAndReserveInclusive() {
        let validator = MinimumAccountReserveValidator(
            available: availableAmount,
            reserve: requiredReserve,
            asset: nativeAsset,
        )

        #expect(throws: TransferAmountCalculatorError.minimumAccountBalanceTooLow(
            nativeAsset,
            requirement: BalanceRequirement(required: requiredReserve, available: 1),
        )) {
            try validator.validate(availableAmount - 1)
        }
        #expect(throws: TransferAmountCalculatorError.minimumAccountBalanceTooLow(
            nativeAsset,
            requirement: BalanceRequirement(required: requiredReserve, available: requiredReserve),
        )) {
            try validator.validate(availableAmount - requiredReserve)
        }
    }

    @Test
    func ignoresWhenAssetIsNonNative() throws {
        let validator = MinimumAccountReserveValidator(
            available: availableAmount,
            reserve: requiredReserve,
            asset: nonNativeAsset,
        )

        try validator.validate(availableAmount)
        try validator.validate(availableAmount - 1)
        try validator.validate(0)
    }
}
