// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemApiClient
import class Gemstone.GemBalanceService
import class Gemstone.GemDeviceApiClient
import class Gemstone.GemDeviceKeyService
import class Gemstone.GemLightningService
import class Gemstone.GemTransactionsService
import class Gemstone.GemWalletSessionService
import GemstonePrimitivesTestKit
import GemstoneServices
import Store
import StoreTestKit

public extension GemLightningService {
    static func mock(
        keystore: LocalKeystore = LocalKeystore.mock(),
        db: DB = .mock(),
    ) -> GemLightningService {
        let session = GemWalletSessionService.mock(store: .mock(db: db))
        let balance = GemBalanceService.mock(db: db)
        return keystore.lightningService(
            api: GemApiClient(provider: StubAlienProvider()),
            storageDir: FileManager.default.temporaryDirectory.appending(path: UUID().uuidString).path(percentEncoded: false),
            balances: balance,
            transactions: GemTransactionsService(
                api: GemDeviceApiClient(provider: StubAlienProvider(), deviceKey: GemDeviceKeyService(store: GemSecureStoreMock())),
                assets: .mock(),
                balance: balance,
                store: GemstoneTransactionStateStore(store: .mock(db: db)),
                names: .mock(),
                walletPreferences: .mock(),
                session: session,
                transactionStatus: GemstoneTransactionStatusService(service: GemTransactionStateServiceMock()),
            ),
            session: session,
            unlockContext: UnlockContextStore(),
        )
    }
}
