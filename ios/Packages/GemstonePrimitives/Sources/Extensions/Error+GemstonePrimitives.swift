// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemErrorText
import enum Gemstone.GemServiceError
import enum Gemstone.GemWalletConnectFailure

extension GemErrorText: @retroactive Error {}
extension GemWalletConnectFailure: @retroactive Error {}

public extension Error {
    var isCancelled: Bool {
        switch self {
        case is CancellationError: true
        case let error as GemServiceError where error == .Cancelled: true
        default: (self as NSError).code == NSURLErrorCancelled
        }
    }
}
