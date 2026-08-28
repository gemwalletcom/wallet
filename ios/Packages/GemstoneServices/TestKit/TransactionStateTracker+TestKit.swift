// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitivesTestKit
import GemstoneServices
import Store
import StoreTestKit

public extension TransactionStateTracker {
    static func mock(transactionStore: TransactionStore = .mock()) -> TransactionStateTracker {
        TransactionStateTracker(
            service: GemTransactionStateServiceMock(store: GemstoneTransactionStateStore(store: transactionStore)),
        )
    }
}
