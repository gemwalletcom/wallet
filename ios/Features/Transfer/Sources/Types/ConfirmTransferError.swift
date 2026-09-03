// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemConfirmError
import GemstoneServices
import Primitives

enum ConfirmTransferError {
    case confirm(GemConfirmError)
    case other(Error)

    init(error: Error) {
        switch error {
        case let error as GemConfirmError where error.hasInfoSheet:
            self = .confirm(error)
        default:
            self = .other(error)
        }
    }

    var displayError: Error {
        switch self {
        case let .confirm(error): error
        case let .other(error): error
        }
    }
}
