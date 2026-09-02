// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemAvatarService
import class Gemstone.GemChainService
import class Gemstone.GemDeviceApiClient
import class Gemstone.GemDeviceKeyService
import class Gemstone.GemExplorerService
import class Gemstone.GemNameService
import class Gemstone.GemOnboardingService
import class Gemstone.GemPreferencesService
import class Gemstone.GemWalletPreferencesService
import class Gemstone.GemWalletService
import class Gemstone.GemWalletSessionService
import GemstonePrimitivesTestKit
import GemstoneServices
import NativeProviderService
import Primitives
import Store
import StoreTestKit

public extension GemOnboardingService {
    static func mock(
        keystore: LocalKeystore = LocalKeystore.mock(),
        walletStore: WalletStore = .mock(),
        sessionStore: GemstoneWalletSessionStore = .mock(),
        addressStore: AddressStore = .mock(),
    ) -> GemOnboardingService {
        let gemWalletStore = GemstoneWalletStore(store: walletStore)
        let session = GemWalletSessionService(store: sessionStore, wallets: gemWalletStore)
        let provider = NativeProvider(url: Constants.apiURL)
        let api = GemDeviceApiClient(provider: provider, baseUrl: Constants.apiURL.absoluteString, deviceKey: GemDeviceKeyService(store: GemSecureStoreMock()))
        return GemOnboardingService(
            wallets: GemWalletService.mock(keystore: keystore, walletStore: walletStore, sessionStore: sessionStore),
            session: session,
            avatars: GemAvatarService(wallets: gemWalletStore, files: GemstoneFileStore(), provider: provider),
            names: GemNameService(api: api, store: GemstoneAddressStore(store: addressStore)),
            chains: GemChainService(),
        )
    }
}

public extension GemWalletService {
    static func mock(
        keystore: LocalKeystore = LocalKeystore.mock(),
        walletStore: WalletStore = .mock(),
        sessionStore: GemstoneWalletSessionStore = .mock(),
    ) -> GemWalletService {
        let gemWalletStore = GemstoneWalletStore(store: walletStore)
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
        )
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
        let store = WalletStore.mock(db: .mock())
        try store.addWallet(wallet)
        return GemWalletSessionService(store: GemstoneWalletSessionStore.mock(), wallets: GemstoneWalletStore(store: store))
    }
}
