// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemAvatarService
import class Gemstone.GemChainService
import class Gemstone.GemDeviceApiClient
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
        let api = GemDeviceApiClient(provider: provider, baseUrl: Constants.apiURL.absoluteString, devicePrivateKey: Data())
        return GemOnboardingService(
            wallets: GemWalletService(
                keystore: keystore.gemKeystore,
                password: GemstoneKeystorePassword(keystore: keystore),
                store: gemWalletStore,
                session: session,
                appPreferences: GemPreferencesService(store: GemPreferencesStoreMock()),
                files: GemstoneFileStore(),
                preferences: GemWalletPreferencesService.mock(),
            ),
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
        return GemWalletService(
            keystore: keystore.gemKeystore,
            password: GemstoneKeystorePassword(keystore: keystore),
            store: gemWalletStore,
            session: GemWalletSessionService(store: sessionStore, wallets: gemWalletStore),
            appPreferences: GemPreferencesService(store: GemPreferencesStoreMock()),
            files: GemstoneFileStore(),
            preferences: GemWalletPreferencesService.mock(),
        )
    }
}

public extension GemWalletSessionService {
    static func mock(
        walletStore: WalletStore = .mock(),
        sessionStore: GemstoneWalletSessionStore = .mock(),
    ) -> GemWalletSessionService {
        GemWalletSessionService(store: sessionStore, wallets: GemstoneWalletStore(store: walletStore))
    }
}
