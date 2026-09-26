// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemSettingsServiceProtocol
import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
import Settings

public extension PreferencesSceneViewModel {
    static func mock(settings: any GemSettingsServiceProtocol = GemSettingsServiceMock()) -> PreferencesSceneViewModel {
        PreferencesSceneViewModel(settings: settings, preferences: .mock())
    }
}
