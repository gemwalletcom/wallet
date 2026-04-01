// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import Primitives

public extension TransactionStateRequest {
    func map() -> GemTransactionStateRequest {
        GemTransactionStateRequest(
            id: id,
            senderAddress: senderAddress,
            createdAt: Int64(createdAt.timeIntervalSince1970),
            blockNumber: Int64(block),
        )
    }
}
