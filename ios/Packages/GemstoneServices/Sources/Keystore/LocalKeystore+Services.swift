// Copyright (c). Gem Wallet. All rights reserved.

public import Gemstone
import Foundation

public extension LocalKeystore {
    func walletService(
        store: any GemWalletStore,
        session: GemWalletSessionService,
        appPreferences: GemPreferencesService,
        files: any GemFileStore,
        preferences: GemWalletPreferencesService,
        explorer: GemExplorerService,
        names: GemNameService,
        avatar: GemAvatarService,
    ) -> GemWalletService {
        GemWalletService(
            keystore: gemKeystore,
            password: password,
            store: store,
            session: session,
            appPreferences: appPreferences,
            files: files,
            preferences: preferences,
            explorer: explorer,
            names: names,
            avatar: avatar,
        )
    }

    func swapService(swapper: GemSwapper, store: any GemSwapStore) -> GemSwapService {
        GemSwapService(
            swapper: swapper,
            keystore: gemKeystore,
            password: password,
            store: store,
        )
    }

    func signMessageService(names: GemNameService, explorer: GemExplorerService) -> GemSignMessageService {
        GemSignMessageService(
            names: names,
            explorer: explorer,
            keystore: gemKeystore,
            password: password,
        )
    }

    func authService(api: GemDeviceApiClient, deviceKey: GemDeviceKeyService) -> GemAuthService {
        GemAuthService(
            api: api,
            keystore: gemKeystore,
            password: password,
            deviceKey: deviceKey,
        )
    }

    private var password: any GemKeystorePassword {
        GemstoneKeystorePassword(keystore: self)
    }
}
