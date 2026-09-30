// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemDeviceKeyService
import GemstonePrimitivesTestKit
import GemstoneServices
import Keychain

public extension GemstoneDevicePlatform {
    @MainActor
    static func mock(keychain: any Keychain) -> GemstoneDevicePlatform {
        GemstoneDevicePlatform(
            preferencesService: GemPreferencesServiceMock(),
            deviceKeyService: GemDeviceKeyService(store: GemSecureStoreMock()),
            keychain: keychain,
        )
    }
}
