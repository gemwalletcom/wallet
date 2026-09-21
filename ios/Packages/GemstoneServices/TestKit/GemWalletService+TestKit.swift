// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemAvatarService
import class Gemstone.GemExplorerService
import class Gemstone.GemNameService
import class Gemstone.GemPaymentService
import class Gemstone.GemPreferencesService
import class Gemstone.GemRecipientService
import class Gemstone.GemSignMessageService
import class Gemstone.GemWalletPreferencesService
import class Gemstone.GemWalletService
import class Gemstone.GemWalletSessionService
import GemstonePrimitives
import GemstonePrimitivesTestKit
import GemstoneServices
import NativeProviderService
import Primitives
import Store
import StoreTestKit

public extension GemWalletService {
    static func mock(
        keystore: LocalKeystore = LocalKeystore.mock(),
        db: DB = .mock(),
        sessionStore: GemstoneWalletSessionStore = .mock(),
    ) -> GemWalletService {
        let gemWalletStore = GemstoneWalletStore(store: WalletStore(db: db))
        let appPreferences = GemPreferencesService(store: GemPreferencesStoreMock())
        return GemWalletService(
            keystore: keystore.gemKeystore,
            password: GemstoneKeystorePassword(keystore: keystore),
            store: gemWalletStore,
            session: GemWalletSessionService(store: sessionStore, wallets: gemWalletStore),
            appPreferences: appPreferences,
            files: GemstoneFileStore(),
            preferences: GemWalletPreferencesService.mock(),
            explorer: GemExplorerService(preferences: appPreferences),
            addresses: GemstoneAddressStore(store: AddressStore(db: db)),
            avatar: GemAvatarService(wallets: gemWalletStore, files: GemstoneFileStore(), provider: NativeProvider()),
        )
    }
}

public extension GemRecipientService {
    static func mock() -> GemRecipientService {
        GemRecipientService(payments: GemPaymentService.mock(), session: GemWalletSessionService.mock())
    }
}

public extension GemWalletSessionService {
    static func mock(
        store: WalletStore = .mock(),
        sessionStore: GemstoneWalletSessionStore = .mock(),
    ) -> GemWalletSessionService {
        GemWalletSessionService(store: sessionStore, wallets: GemstoneWalletStore(store: store))
    }

    static func mock(wallet: Wallet) throws -> GemWalletSessionService {
        let service = try GemWalletSessionService.mock(store: .mock(db: .mockWithWallets([wallet])))
        try service.setCurrent(walletId: wallet.id)
        return service
    }
}

public extension GemSignMessageService {
    static func mock(keystore: LocalKeystore = LocalKeystore.mock()) -> GemSignMessageService {
        GemSignMessageService(
            names: GemNameService.mock(),
            explorer: GemExplorerService(preferences: GemPreferencesService(store: GemPreferencesStoreMock())),
            keystore: keystore.gemKeystore,
            password: GemstoneKeystorePassword(keystore: keystore),
        )
    }
}
