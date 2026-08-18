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
    let tempoToken = Asset.tempoUSDC()
    let fee = BigInt(5_000)

    @Test
    func validateSuccess() throws {
        let result = try service.validate(
            transferData: .mock(type: .transfer(asset), amount: .exact(BigInt(10_000_000))),
            availableValue: BigInt(100_000_000),
            feeAsset: asset.feeAsset,
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
            feeAsset: asset.feeAsset,
            assetFeeBalance: BigInt(1_000_000_000),
            fee: fee,
        ).get()

        #expect(result == TransferAmount(value: BigInt(1_000_000_000) - fee, networkFee: fee, useMaxAmount: true))
    }

    @Test
    func insufficientBalanceReportsSendAsset() {
        let result = service.validate(
            transferData: .mock(type: .transfer(asset), amount: .exact(BigInt(20_000_000))),
            availableValue: BigInt(10_000_000),
            feeAsset: asset.feeAsset,
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
            transferData: .mock(type: .transfer(asset), amount: .exact(BigInt(10_000_000))),
            availableValue: BigInt(10_000_000),
            feeAsset: asset.feeAsset,
            assetFeeBalance: BigInt(10_000_000),
            fee: fee,
        )

        #expect(result == .failure(.minimumAccountBalanceTooLow(
            asset,
            requirement: BalanceRequirement(required: BigInt(890_880), available: BigInt(-5_000)),
        )))
    }

    @Test
    func insufficientNetworkFeeReportsFeeAsset() {
        let result = service.validate(
            transferData: .mock(type: .transfer(token), amount: .exact(BigInt(10_000_000))),
            availableValue: BigInt(10_000_000),
            feeAsset: token.feeAsset,
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
            feeAsset: asset.feeAsset,
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
        let result = try validateTempoToken(amount: .exact(BigInt(400)), feeBalance: BigInt(1_000)).get()
        #expect(result == TransferAmount(value: BigInt(400), networkFee: BigInt(500), useMaxAmount: false))

        let maxResult = try validateTempoToken(amount: .max(BigInt(1_000)), feeBalance: BigInt(1_000)).get()
        #expect(maxResult == TransferAmount(value: BigInt(500), networkFee: BigInt(500), useMaxAmount: true))

        #expect(validateTempoToken(amount: .exact(BigInt(400)), feeBalance: BigInt(100)) == .failure(.insufficientNetworkFee(
            tempoToken,
            requirement: BalanceRequirement(required: BigInt(500), available: BigInt(100)),
        )))

        #expect(validateTempoToken(amount: .exact(BigInt(600)), feeBalance: BigInt(1_000)) == .failure(.insufficientBalance(
            tempoToken,
            requirement: BalanceRequirement(required: BigInt(1_100), available: BigInt(1_000)),
        )))
    }

    private func validateTempoToken(amount: TransferAmountValue, feeBalance: BigInt) -> TransferAmountValidation {
        service.validate(
            transferData: .mock(type: .transfer(tempoToken), amount: amount),
            availableValue: BigInt(1_000),
            feeAsset: tempoToken,
            assetFeeBalance: feeBalance,
            fee: BigInt(500),
        )
    }
}
