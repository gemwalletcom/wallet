// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemConfirmError
import enum Gemstone.GemConfirmErrorDisplay

enum ConfirmTransferError {
    case confirm(GemConfirmError)
    case other(Error)

    init(error: Error) {
        switch error {
        case let error as GemConfirmError where error.display().hasInfoSheet():
            self = .confirm(error)
        default:
            self = .other(error)
        }
    }

    var hasInfoSheet: Bool {
        switch self {
        case .confirm: true
        case .other: false
        }
    }

    var displayError: Error {
        switch self {
        case let .confirm(error): error
        case let .other(error): error
        }
    }
}

extension GemConfirmErrorDisplay: @retroactive Error {}
