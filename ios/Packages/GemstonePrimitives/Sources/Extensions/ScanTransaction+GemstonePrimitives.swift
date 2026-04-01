// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Gemstone.ScanTransaction {
    func map() throws -> Primitives.ScanTransaction {
        Primitives.ScanTransaction(
            isMalicious: isMalicious,
            isMemoRequired: isMemoRequired,
        )
    }
}
