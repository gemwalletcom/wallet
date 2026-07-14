// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Security

public extension Error {
    var isCancelled: Bool {
        switch self {
        case _ where (self is CancellationError) == true:
            true
        case _ where (self as NSError).code == NSURLErrorCancelled:
            true
        default:
            false
        }
    }

    var isAuthenticationCancelled: Bool {
        (self as NSError).code == Int(errSecUserCanceled)
    }
}
