// Copyright (c). Gem Wallet. All rights reserved.

import AppService
import Foundation
import class Gemstone.GemApiClient
import class Gemstone.GemConfigService
import NativeProviderService
import Preferences
import PreferencesTestKit
import Primitives
import PrimitivesTestKit

public extension ConfigService {
    static func mock(
        configPreferences: ConfigPreferences = .mock(),
        service: GemConfigService = .mock(),
    ) -> ConfigService {
        ConfigService(
            configPreferences: configPreferences,
            service: service,
        )
    }
}

public extension GemConfigService {
    static func mock() -> GemConfigService {
        GemConfigService(
            api: GemApiClient(
                provider: NativeProvider(url: Constants.apiURL, requestInterceptor: EmptyRequestInterceptor()),
                baseUrl: Constants.apiURL.absoluteString,
            ),
        )
    }
}
