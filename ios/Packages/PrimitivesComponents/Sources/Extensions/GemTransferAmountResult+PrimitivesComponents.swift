// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemConfirmError
import struct Gemstone.GemTransferAmount
import enum Gemstone.GemTransferAmountResult
import Foundation

public typealias TransferAmountValidation = Result<GemTransferAmount, GemConfirmError>

public extension GemTransferAmountResult {
    func map() -> TransferAmountValidation {
        switch self {
        case let .amount(amount): .success(amount)
        case let .error(error): .failure(error)
        }
    }
}
