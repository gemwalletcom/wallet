// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemPreferencesService
import protocol Gemstone.GemPreferencesServiceProtocol
import GemstonePrimitivesTestKit
import Preferences

public extension ObservablePreferences {
    static func mock(
        preferencesService: any GemPreferencesServiceProtocol = GemPreferencesService(store: GemPreferencesStoreMock()),
        isPerpetualEnabled: Bool = true,
    ) -> ObservablePreferences {
        let observable = ObservablePreferences(preferencesService: preferencesService)
        observable.isPerpetualEnabled = isPerpetualEnabled
        return observable
    }
}
