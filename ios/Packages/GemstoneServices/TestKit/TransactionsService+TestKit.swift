// Copyright (c). Gem Wallet. All rights reserved.

import PrimitivesTestKit
import protocol Gemstone.GemTransactionsServiceProtocol
import GemstonePrimitivesTestKit
import Store
import StoreTestKit
import GemstoneServices

public extension TransactionsService {
    static func mock(
        service: any GemTransactionsServiceProtocol = GemTransactionsServiceMock(),
        transactionStore: TransactionStore = .mock(),
    ) -> TransactionsService {
        TransactionsService(
            service: service,
            transactionStore: transactionStore,
        )
    }
}
