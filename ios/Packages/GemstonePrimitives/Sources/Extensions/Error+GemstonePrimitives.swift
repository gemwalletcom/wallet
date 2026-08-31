// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemServiceError

public extension Error {
    var isCancelled: Bool {
        switch self {
        case is CancellationError: true
        case let error as GemServiceError where error == .Cancelled: true
        default: (self as NSError).code == NSURLErrorCancelled
        }
    }
}
