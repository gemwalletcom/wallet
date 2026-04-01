// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import Primitives

public extension TransactionPreloadInput {
    func map() throws -> GemTransactionPreloadInput {
        try GemTransactionPreloadInput(
            inputType: inputType.map(),
            senderAddress: senderAddress,
            destinationAddress: destinationAddress,
        )
    }
}
