// Copyright (c). Gem Wallet. All rights reserved.

import AssetsService
import AssetsServiceTestKit
import protocol Gemstone.GemTransactionsServiceProtocol
import GemstonePrimitivesTestKit
import Store
import StoreTestKit
import TransactionsService

public extension TransactionsService {
    static func mock(
        provider: any GemTransactionsServiceProtocol = GemTransactionsServiceMock(),
        transactionStore: TransactionStore = .mock(),
        assetsService: AssetsService = .mock(),
        addressStore: AddressStore = .mock(),
    ) -> TransactionsService {
        TransactionsService(
            provider: provider,
            transactionStore: transactionStore,
            assetsService: assetsService,
            addressStore: addressStore,
        )
    }
}
