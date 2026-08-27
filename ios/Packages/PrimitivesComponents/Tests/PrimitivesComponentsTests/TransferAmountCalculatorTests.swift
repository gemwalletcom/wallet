// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit
import Testing
import Validators

struct TransferAmountCalculatorTests {
    let service = TransferAmountCalculator()
    let asset = Asset.mockSolana()
    let token = Asset.mockSolanaUSDC()
    let tempoToken = Asset.mockTempoUSDC()
    let fee = BigInt(5000)

    @Test
    func validateSuccess() throws {
        let result = try service.validate(
            transferData: .mock(type: .transfer(asset), value: BigInt(10_000_000)),
            availableValue: BigInt(100_000_000),
            feeAsset: asset,
            assetFeeBalance: BigInt(100_000_000),
            fee: fee,
        ).get()

        #expect(result == TransferAmount(value: BigInt(10_000_000), networkFee: fee, useMaxAmount: false))
    }

    @Test
    func validateMaxAmountSubtractsFee() throws {
        let result = try service.validate(
            transferData: .mock(type: .transfer(asset), value: BigInt(1_000_000_000), useMaxAmount: true),
            availableValue: BigInt(1_000_000_000),
            feeAsset: asset,
            assetFeeBalance: BigInt(1_000_000_000),
            fee: fee,
        ).get()

        #expect(result == TransferAmount(value: BigInt(1_000_000_000) - fee, networkFee: fee, useMaxAmount: true))
    }

    @Test
    func insufficientBalanceReportsSendAsset() {
        let result = service.validate(
            transferData: .mock(type: .transfer(asset), value: BigInt(20_000_000)),
            availableValue: BigInt(10_000_000),
            feeAsset: asset,
            assetFeeBalance: BigInt(10_000_000),
            fee: fee,
        )

        #expect(result == .failure(.insufficientBalance(
            asset,
            requirement: BalanceRequirement(required: BigInt(20_005_000), available: BigInt(10_000_000)),
        )))
    }

    @Test
    func nearMaxReportsMinimumAccountBalance() {
        let result = service.validate(
            transferData: .mock(type: .transfer(asset), value: BigInt(10_000_000)),
            availableValue: BigInt(10_000_000),
            feeAsset: asset,
            assetFeeBalance: BigInt(10_000_000),
            fee: fee,
        )

        #expect(result == .failure(.minimumAccountBalanceTooLow(
            asset,
            requirement: BalanceRequirement(required: BigInt(890_880), available: BigInt(-5000)),
        )))
    }

    @Test
    func insufficientNetworkFeeReportsFeeAsset() {
        let result = service.validate(
            transferData: .mock(type: .transfer(token), value: BigInt(10_000_000)),
            availableValue: BigInt(10_000_000),
            feeAsset: asset,
            assetFeeBalance: BigInt(1000),
            fee: fee,
        )

        #expect(result == .failure(.insufficientNetworkFee(
            asset,
            requirement: BalanceRequirement(required: fee, available: BigInt(1000)),
        )))
    }

    @Test
    func minimumAccountBalanceReportsSendAsset() {
        let result = service.validate(
            transferData: .mock(type: .transfer(asset), value: BigInt(10_000_000)),
            availableValue: BigInt(10_500_000),
            feeAsset: asset,
            assetFeeBalance: BigInt(10_500_000),
            fee: fee,
        )

        #expect(result == .failure(.minimumAccountBalanceTooLow(
            asset,
            requirement: BalanceRequirement(required: BigInt(890_880), available: BigInt(495_000)),
        )))
    }

    @Test
    func validateTempoTokenFeeAsset() throws {
        let result = try validateTempoToken(value: BigInt(400), feeBalance: BigInt(1000)).get()
        #expect(result == TransferAmount(value: BigInt(400), networkFee: BigInt(500), useMaxAmount: false))

        let maxResult = try validateTempoToken(value: BigInt(1000), useMaxAmount: true, feeBalance: BigInt(1000)).get()
        #expect(maxResult == TransferAmount(value: BigInt(500), networkFee: BigInt(500), useMaxAmount: true))

        #expect(validateTempoToken(value: BigInt(400), feeBalance: BigInt(100)) == .failure(.insufficientNetworkFee(
            tempoToken,
            requirement: BalanceRequirement(required: BigInt(500), available: BigInt(100)),
        )))

        #expect(validateTempoToken(value: BigInt(600), feeBalance: BigInt(1000)) == .failure(.insufficientBalance(
            tempoToken,
            requirement: BalanceRequirement(required: BigInt(1100), available: BigInt(1000)),
        )))
    }

    private func validateTempoToken(value: BigInt, useMaxAmount: Bool = false, feeBalance: BigInt) -> TransferAmountValidation {
        service.validate(
            transferData: .mock(type: .transfer(tempoToken), value: value, useMaxAmount: useMaxAmount),
            availableValue: BigInt(1000),
            feeAsset: tempoToken,
            assetFeeBalance: feeBalance,
            fee: BigInt(500),
        )
    }
}
