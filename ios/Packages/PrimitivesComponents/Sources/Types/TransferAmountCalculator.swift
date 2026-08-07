// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import GemstonePrimitives
import Primitives
import Validators

public typealias TransferAmountValidation = Result<TransferAmount, TransferAmountCalculatorError>

public struct TransferAmountCalculator {
    public init() {}

    public func validate(
        transferData: TransferData,
        availableValue: BigInt,
        assetFeeBalance: BigInt,
        fee: BigInt,
    ) -> TransferAmountValidation {
        let asset = transferData.type.asset
        do {
            return try .success(
                TransferAmount.calculate(
                    transferData: transferData,
                    availableValue: availableValue,
                    assetFeeBalance: assetFeeBalance,
                    fee: fee,
                )
            )
        } catch let error as TransferAmountError {
            return .failure(TransferAmountCalculatorError(error, asset: asset, assetFee: asset.feeAsset))
        } catch {
            return .failure(.insufficientBalance(asset, requirement: BalanceRequirement(required: transferData.value, available: availableValue)))
        }
    }
}
