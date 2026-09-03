// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemConfirmError
import enum Gemstone.GemTransferAmountResult
import Foundation
import GemstonePrimitives
import Primitives

public typealias TransferAmountValidation = Result<Primitives.TransferAmount, GemConfirmError>

public extension GemTransferAmountResult {
    func map() -> TransferAmountValidation {
        switch self {
        case let .amount(amount):
            .success(Primitives.TransferAmount(
                value: amount.value,
                networkFee: amount.networkFee,
                useMaxAmount: amount.isMaxAmount,
            ))
        case let .error(error):
            .failure(error)
        }
    }
}
