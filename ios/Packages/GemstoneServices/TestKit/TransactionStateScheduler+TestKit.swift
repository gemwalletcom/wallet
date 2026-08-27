// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitivesTestKit
import GemstoneServices
import Store
import StoreTestKit

public extension TransactionStateScheduler {
    static func mock(transactionStore: TransactionStore = .mock()) -> TransactionStateScheduler {
        TransactionStateScheduler(
            service: TransactionStateService(
                service: GemTransactionStateServiceMock(store: GemstoneTransactionStateStore(store: transactionStore)) { _, _ in nil },
            ),
        )
    }
}
