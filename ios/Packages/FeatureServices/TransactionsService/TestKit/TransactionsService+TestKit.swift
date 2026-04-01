// Copyright (c). Gem Wallet. All rights reserved.

import AssetsServiceTestKit
import Foundation
import GemAPITestKit
import StoreTestKit
import TransactionsService

public extension TransactionsService {
    static func mock() -> TransactionsService {
        TransactionsService(
            provider: GemAPITransactionServiceMock(),
            transactionStore: .mock(),
            assetsService: .mock(),
            walletStore: .mock(),
            addressStore: .mock(),
        )
    }
}
