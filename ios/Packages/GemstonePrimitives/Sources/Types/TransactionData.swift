// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public struct TransactionData: Sendable {
    public let fee: Fee
    public let metadata: GemTransactionLoadMetadata

    public init(
        fee: Fee,
        metadata: GemTransactionLoadMetadata = .none,
    ) {
        self.fee = fee
        self.metadata = metadata
    }
}
