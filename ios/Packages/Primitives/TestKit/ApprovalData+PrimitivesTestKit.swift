// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension ApprovalData {
    static func mock(
    ) -> ApprovalData {
        ApprovalData(token: "", spender: "", value: "0", isUnlimited: false)
    }
}
