// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import enum Gemstone.GemTransferAmountError
import enum Gemstone.GemTransferAmountResult
import Foundation
import GemstonePrimitives
import Primitives
import Validators

public typealias TransferAmountValidation = Result<Primitives.TransferAmount, TransferAmountCalculatorError>

public extension GemTransferAmountResult {
    func map() -> TransferAmountValidation {
        switch self {
        case let .amount(amount):
            .success(Primitives.TransferAmount(
                value: BigInt(core: amount.value),
                networkFee: BigInt(core: amount.networkFee),
                useMaxAmount: amount.isMaxAmount,
            ))
        case let .error(error, asset):
            .failure(error.calculatorError(asset: asset.map()))
        }
    }
}

private extension GemTransferAmountError {
    func calculatorError(asset: Primitives.Asset) -> TransferAmountCalculatorError {
        switch self {
        case .InsufficientBalance: .insufficientBalance(asset, requirement: requirement)
        case .InsufficientNetworkFee: .insufficientNetworkFee(asset, requirement: requirement)
        case .MinimumAccountBalanceTooLow: .minimumAccountBalanceTooLow(asset, requirement: requirement)
        }
    }
}
