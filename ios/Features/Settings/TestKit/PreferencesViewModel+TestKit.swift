// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemSettingsServiceProtocol
import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
import Settings

public extension PreferencesViewModel {
    static func mock(settings: any GemSettingsServiceProtocol = GemSettingsServiceMock()) -> PreferencesViewModel {
        PreferencesViewModel(settings: settings, preferences: .mock())
    }
}
