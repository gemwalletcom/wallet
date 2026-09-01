// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import GemstonePrimitives
import Primitives

public extension TransferAmount {
    static func calculate(
        transferData: TransferData,
        availableValue: BigInt,
        feeAssetId: Primitives.AssetId,
        assetFeeBalance: BigInt,
        fee: BigInt,
        amountService: GemAmountService,
    ) throws -> TransferAmount {
        let input = try GemTransferAmountInput(
            inputType: transferData.type,
            value: transferData.value.description,
            availableValue: availableValue.description,
            feeAsset: feeAssetId.identifier,
            feeAssetBalance: assetFeeBalance.description,
            fee: fee.description,
            isMaxAmount: transferData.useMaxAmount,
            minimumValue: transferData.minimumValue?.description,
        )
        do {
            return try amountService.calculate(input: input).map()
        } catch let error as GemTransferAmountError {
            throw try error.map()
        }
    }
}

public extension GemTransferAmount {
    func map() throws -> TransferAmount {
        try TransferAmount(
            value: BigInt.from(string: value),
            networkFee: BigInt.from(string: networkFee),
            useMaxAmount: isMaxAmount,
        )
    }
}

public extension GemTransferAmountError {
    func map() throws -> TransferAmountError {
        switch self {
        case let .InsufficientBalance(assetId, required, available):
            try .insufficientBalance(assetId: AssetId(id: assetId), requirement: Self.requirement(required, available))
        case let .InsufficientNetworkFee(assetId, required, available):
            try .insufficientNetworkFee(assetId: AssetId(id: assetId), requirement: Self.requirement(required, available))
        case let .MinimumAccountBalanceTooLow(assetId, required, available):
            try .minimumAccountBalanceTooLow(assetId: AssetId(id: assetId), requirement: Self.requirement(required, available))
        }
    }

    private static func requirement(_ required: String, _ available: String) throws -> BalanceRequirement {
        try BalanceRequirement(
            required: BigInt.from(string: required),
            available: BigInt.from(string: available),
        )
    }
}
