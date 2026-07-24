// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Primitives
import PrimitivesTestKit
import Testing
import Validators

@testable import PrimitivesComponents

struct TransferAmountCalculatorTests {
    let service = TransferAmountCalculator()
    let asset = Asset.mockSolana()
    let token = Asset.mockSolanaUSDC()
    let fee = BigInt(5_000)

    @Test
    func validateSuccess() throws {
        let result = try service.validate(
            transferData: .mock(type: .transfer(asset), amount: .exact(BigInt(10_000_000))),
            availableValue: BigInt(100_000_000),
            assetFeeBalance: BigInt(100_000_000),
            fee: fee,
        ).get()

        #expect(result == TransferAmount(value: BigInt(10_000_000), networkFee: fee, useMaxAmount: false))
    }

    @Test
    func validateMaxAmountSubtractsFee() throws {
        let result = try service.validate(
            transferData: .mock(type: .transfer(asset), amount: .max(BigInt(1_000_000_000))),
            availableValue: BigInt(1_000_000_000),
            assetFeeBalance: BigInt(1_000_000_000),
            fee: fee,
        ).get()

        #expect(result == TransferAmount(value: BigInt(1_000_000_000) - fee, networkFee: fee, useMaxAmount: true))
    }

    @Test
    func insufficientBalanceReportsSendAsset() {
        let result = service.validate(
            transferData: .mock(type: .transfer(asset), amount: .exact(BigInt(10_000_000))),
            availableValue: BigInt(10_000_000),
            assetFeeBalance: BigInt(10_000_000),
            fee: fee,
        )

        #expect(result == .failure(.insufficientBalance(
            asset,
            requirement: BalanceRequirement(required: BigInt(10_005_000), available: BigInt(10_000_000)),
        )))
    }

    @Test
    func insufficientNetworkFeeReportsFeeAsset() {
        let result = service.validate(
            transferData: .mock(type: .transfer(token), amount: .exact(BigInt(10_000_000))),
            availableValue: BigInt(10_000_000),
            assetFeeBalance: BigInt(1_000),
            fee: fee,
        )

        #expect(result == .failure(.insufficientNetworkFee(
            token.feeAsset,
            requirement: BalanceRequirement(required: fee, available: BigInt(1_000)),
        )))
    }

    @Test
    func minimumAccountBalanceReportsSendAsset() {
        let result = service.validate(
            transferData: .mock(type: .transfer(asset), amount: .exact(BigInt(10_000_000))),
            availableValue: BigInt(10_500_000),
            assetFeeBalance: BigInt(10_500_000),
            fee: fee,
        )

        #expect(result == .failure(.minimumAccountBalanceTooLow(
            asset,
            requirement: BalanceRequirement(required: BigInt(890_880), available: BigInt(495_000)),
        )))
    }
}
