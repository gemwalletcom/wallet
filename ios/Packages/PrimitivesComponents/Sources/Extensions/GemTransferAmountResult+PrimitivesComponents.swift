// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemConfirmError
import struct Gemstone.GemTransferAmount
import enum Gemstone.GemTransferAmountResult

public typealias TransferAmountValidation = Result<GemTransferAmount, GemConfirmError>

public extension GemTransferAmountResult {
    func toPrimitives() -> TransferAmountValidation {
        switch self {
        case let .amount(amount): .success(amount)
        case let .error(error): .failure(error)
        }
    }
}
