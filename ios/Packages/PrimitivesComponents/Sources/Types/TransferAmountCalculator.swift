// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import class Gemstone.GemAmountService
import GemstonePrimitives
import Primitives
import Validators
import struct Gemstone.GemTransferData

public typealias TransferAmountValidation = Result<TransferAmount, TransferAmountCalculatorError>

public struct TransferAmountCalculator {
    private let amountService: GemAmountService

    public init(amountService: GemAmountService) {
        self.amountService = amountService
    }

    public func validate(
        transferData: GemTransferData,
        availableValue: BigInt,
        feeAsset: Asset,
        assetFeeBalance: BigInt,
        fee: BigInt,
    ) -> TransferAmountValidation {
        let asset = transferData.inputType.asset
        do {
            return try .success(
                TransferAmount.calculate(
                    transferData: transferData,
                    availableValue: availableValue,
                    feeAssetId: feeAsset.id,
                    assetFeeBalance: assetFeeBalance,
                    fee: fee,
                    amountService: amountService,
                )
            )
        } catch let error as TransferAmountError {
            return .failure(TransferAmountCalculatorError(error, asset: asset, assetFee: feeAsset))
        } catch {
            return .failure(.insufficientBalance(asset, requirement: BalanceRequirement(required: BigInt(core: transferData.value), available: availableValue)))
        }
    }
}
