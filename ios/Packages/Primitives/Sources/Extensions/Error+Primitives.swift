// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Security

public extension Error {
    var isAuthenticationCancelled: Bool {
        (self as NSError).code == Int(errSecUserCanceled)
    }
}
