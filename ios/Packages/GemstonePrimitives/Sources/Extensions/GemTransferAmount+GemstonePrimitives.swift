// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import Primitives

public extension TransferAmount {
    static func calculate(
        transferData: TransferData,
        availableValue: BigInt,
        feeAsset: Asset,
        assetFeeBalance: BigInt,
        fee: BigInt,
    ) throws -> TransferAmount {
        let isMaxAmount = switch transferData.amount {
        case .exact: false
        case .max: true
        }
        let input = try GemTransferAmountInput(
            inputType: transferData.type.map(),
            value: transferData.value.description,
            availableValue: availableValue.description,
            feeAsset: feeAsset.map(),
            feeAssetBalance: assetFeeBalance.description,
            fee: fee.description,
            isMaxAmount: isMaxAmount,
            minimumValue: transferData.minimumValue?.description,
        )
        do {
            return try Gemstone.calculateTransferAmount(input: input).map()
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
