// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemAppUpdateServiceProtocol
import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
import Settings

public extension AboutUsSceneViewModel {
    static func mock(
        service: any GemAppUpdateServiceProtocol = GemAppUpdateServiceMock(),
        preferences: ObservablePreferences = .mock(),
    ) -> AboutUsSceneViewModel {
        AboutUsSceneViewModel(preferences: preferences, service: service)
    }
}
