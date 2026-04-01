// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemTransactionData {
    func map() throws -> Primitives.TransactionData {
        try TransactionData(
            fee: fee.map(),
            metadata: metadata.map(),
        )
    }
}
