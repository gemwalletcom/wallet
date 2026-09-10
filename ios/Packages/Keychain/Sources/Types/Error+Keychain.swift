// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public extension Error {
    var isKeychainUserCancelled: Bool {
        (self as? Status) == .userCanceled
    }
}
