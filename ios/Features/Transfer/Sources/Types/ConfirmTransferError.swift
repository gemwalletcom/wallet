// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemConfirmError
import GemstonePrimitives
import GemstoneServices
import Foundation
import Primitives

enum ConfirmTransferError {
    case confirm(GemConfirmError)
    case chain(ChainCoreError)
    case other(Error)

    init(error: Error) {
        switch error {
        case let error as GemConfirmError where error.hasInfoSheet:
            self = .confirm(error)
        default:
            switch ChainCoreError.fromError(error) {
            case let .some(chainError): self = .chain(chainError)
            case .none: self = .other(error)
            }
        }
    }

    var displayError: Error {
        switch self {
        case let .confirm(error): error
        case let .chain(error): error
        case let .other(error): error
        }
    }
}
