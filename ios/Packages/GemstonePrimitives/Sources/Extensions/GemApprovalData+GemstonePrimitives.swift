// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemApprovalData {
    func map() -> ApprovalData {
        ApprovalData(
            token: token,
            spender: spender,
            value: value,
            isUnlimited: isUnlimited,
        )
    }
}

public extension ApprovalData {
    func map() -> GemApprovalData {
        GemApprovalData(
            token: token,
            spender: spender,
            value: value,
            isUnlimited: isUnlimited,
        )
    }
}
