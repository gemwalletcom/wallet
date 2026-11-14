// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Security

public extension Error {
    var isAuthenticationCancelled: Bool {
        let error = self as NSError
        return error.domain == NSOSStatusErrorDomain && error.code == Int(errSecUserCanceled)
    }
}
