// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Primitives
import Validators

public typealias TransferAmountValidation = Result<TransferAmount, TransferAmountCalculatorError>

public struct TransferAmountCalculator {
    public init() {}

    public func validate(input: TransferAmountInput) -> TransferAmountValidation {
        do {
            return try .success(calculate(input: input))
        } catch {
            return .failure(error)
        }
    }

    public func validateNetworkFee(_ feeBalance: BigInt, feeAssetId: AssetId) throws(TransferAmountCalculatorError) {
        if [Chain.hyperCore, Chain.tron].contains(feeAssetId.chain) {
            return
        }
        if feeBalance.isZero, feeAssetId.type == .native {
            throw TransferAmountCalculatorError.insufficientNetworkFee(feeAssetId.chain.asset, requirement: nil)
        }
    }

    func calculate(input: TransferAmountInput) throws(TransferAmountCalculatorError) -> TransferAmount {
        let amount = try calculateAmount(input: input)
        if let minimumValue = input.minimumValue, amount.value < minimumValue {
            throw TransferAmountCalculatorError.insufficientBalance(
                input.asset,
                requirement: BalanceRequirement(required: minimumValue, available: amount.value),
            )
        }
        return amount
    }

    private func calculateAmount(input: TransferAmountInput) throws(TransferAmountCalculatorError) -> TransferAmount {
        if input.assetBalance.available == 0, !input.ignoreValueCheck {
            guard input.fee.isZero else {
                let required = input.value + (input.asset == input.assetFee ? input.fee : .zero)
                throw TransferAmountCalculatorError.insufficientBalance(
                    input.asset,
                    requirement: BalanceRequirement(required: required, available: input.assetBalance.available),
                )
            }
        }

        if input.ignoreValueCheck {
            if input.assetFeeBalance.available < input.fee, input.assetFee.chain != .hyperCore {
                throw TransferAmountCalculatorError.insufficientNetworkFee(
                    input.assetFee,
                    requirement: BalanceRequirement(required: input.fee, available: input.assetFeeBalance.available),
                )
            }
            return TransferAmount(value: input.value, networkFee: input.fee, useMaxAmount: false)
        }

        if input.availableValue < input.value {
            throw TransferAmountCalculatorError.insufficientBalance(
                input.asset,
                requirement: BalanceRequirement(required: input.value, available: input.availableValue),
            )
        }

        if input.assetFeeBalance.available < input.fee {
            throw TransferAmountCalculatorError.insufficientNetworkFee(
                input.assetFee,
                requirement: BalanceRequirement(required: input.fee, available: input.assetFeeBalance.available),
            )
        }

        if !input.canChangeValue,
           input.asset == input.assetFee,
           input.availableValue < input.value + input.fee
        {
            throw TransferAmountCalculatorError.insufficientBalance(
                input.asset,
                requirement: BalanceRequirement(required: input.value + input.fee, available: input.availableValue),
            )
        }

        // max value transfer
        if input.assetBalance.available == input.value {
            if input.asset == input.asset.feeAsset, input.canChangeValue {
                return TransferAmount(
                    value: input.assetBalance.available - input.fee,
                    networkFee: input.fee,
                    useMaxAmount: true,
                )
            }
            return TransferAmount(value: input.assetBalance.available, networkFee: input.fee, useMaxAmount: true)
        }
        if input.asset.type == .native, input.asset.chain.minimumAccountBalance > 0,
           (input.availableValue - input.value - input.fee).isBetween(-BigInt.MAX_256, and: input.asset.chain.minimumAccountBalance)
        {
            throw TransferAmountCalculatorError.minimumAccountBalanceTooLow(
                input.asset,
                requirement: BalanceRequirement(
                    required: input.asset.chain.minimumAccountBalance,
                    available: input.availableValue - input.value - input.fee,
                ),
            )
        }

        let useMaxAmount = input.availableValue == input.value

        return TransferAmount(value: input.value, networkFee: input.fee, useMaxAmount: useMaxAmount)
    }
}
