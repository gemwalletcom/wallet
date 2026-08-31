// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemConfirmError
import GemstonePrimitives
import GemstoneServices
import Foundation
import Primitives
import Validators

enum ConfirmTransferError {
    case amount(TransferAmountCalculatorError)
    case scan(GemConfirmError)
    case chain(ChainCoreError)
    case other(Error)

    init(error: Error) {
        switch error {
        case let error as TransferAmountCalculatorError:
            self = .amount(error)
        case let error as GemConfirmError where error.isScanRejection:
            self = .scan(error)
        default:
            switch ChainCoreError.fromError(error) {
            case let .some(chainError): self = .chain(chainError)
            case .none: self = .other(error)
            }
        }
    }

    var displayError: Error {
        switch self {
        case let .amount(error): error
        case let .scan(error): error
        case let .chain(error): error
        case let .other(error): error
        }
    }
}
