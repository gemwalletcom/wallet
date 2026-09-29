// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

@inlinable
public func debugLog(_ message: @autoclosure () -> String) {
    #if DEBUG
        NSLog("%@", message())
    #endif
}
