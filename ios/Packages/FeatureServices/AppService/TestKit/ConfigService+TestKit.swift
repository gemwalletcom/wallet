// Copyright (c). Gem Wallet. All rights reserved.

import AppService
import Foundation
import GemAPI
import GemAPITestKit
import Preferences
import PreferencesTestKit
import Primitives
import PrimitivesTestKit

public extension ConfigService {
    static func mock(
        configPreferences: ConfigPreferences = .mock(),
        apiService: any GemAPIConfigService = GemAPIConfigServiceMock(config: .mock()),
    ) -> ConfigService {
        ConfigService(
            configPreferences: configPreferences,
            apiService: apiService,
        )
    }
}
