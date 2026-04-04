// Copyright (c). Gem Wallet. All rights reserved.

import AssetsService
import AssetsServiceTestKit
import GemAPI
import GemAPITestKit
import Store
import StoreTestKit
import TransactionsService

public extension TransactionsService {
    static func mock(
        provider: any GemAPITransactionService = GemAPITransactionServiceMock(),
        transactionStore: TransactionStore = .mock(),
        assetsService: AssetsService = .mock(),
        walletStore: WalletStore = .mock(),
        addressStore: AddressStore = .mock(),
    ) -> TransactionsService {
        TransactionsService(
            provider: provider,
            transactionStore: transactionStore,
            assetsService: assetsService,
            walletStore: walletStore,
            addressStore: addressStore,
        )
    }
}
