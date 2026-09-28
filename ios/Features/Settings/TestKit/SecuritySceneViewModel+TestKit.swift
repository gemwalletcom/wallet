// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemSettingsServiceProtocol
import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
import Settings

public extension SecuritySceneViewModel {
    static func mock(
        service: any BiometryAuthenticatable = BiometryAuthenticationMock(),
        settings: any GemSettingsServiceProtocol = GemSettingsServiceMock(),
        preferences: ObservablePreferences = .mock(),
    ) -> SecuritySceneViewModel {
        SecuritySceneViewModel(service: service, settings: settings, preferences: preferences)
    }
}
