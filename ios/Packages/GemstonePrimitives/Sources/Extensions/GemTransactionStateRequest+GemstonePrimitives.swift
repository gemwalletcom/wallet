// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import Primitives

public extension TransactionStateRequest {
    func map() -> GemTransactionStateRequest {
        GemTransactionStateRequest(
            id: id,
            senderAddress: senderAddress,
            createdAt: createdAt.millisecondsSince1970,
            blockNumber: Int64(block),
            swapProvider: try? swapProvider?.map(),
        )
    }
}
