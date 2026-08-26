// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

public extension Primitives.ApprovalData {
    static func mock() -> Primitives.ApprovalData {
        Primitives.ApprovalData(
            token: "0x",
            spender: "0x",
            value: "1000000000000000000",
            isUnlimited: false,
        )
    }
}
