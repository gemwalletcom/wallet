// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemPreferencesServiceProtocol
import GemstonePrimitivesTestKit
import GemstoneServices

public extension PushNotificationEnablerService {
    static func mock(preferencesService: any GemPreferencesServiceProtocol = GemPreferencesServiceMock()) -> Self {
        PushNotificationEnablerService(preferencesService: preferencesService)
    }
}
