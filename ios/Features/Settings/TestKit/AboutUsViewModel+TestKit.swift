// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemAppUpdateServiceProtocol
import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
import Settings

public extension AboutUsViewModel {
    static func mock(
        service: any GemAppUpdateServiceProtocol = GemAppUpdateServiceMock(),
        preferences: ObservablePreferences = .mock(),
    ) -> AboutUsViewModel {
        AboutUsViewModel(preferences: preferences, service: service)
    }
}
