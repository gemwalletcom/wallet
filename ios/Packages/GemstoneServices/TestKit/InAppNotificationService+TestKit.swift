// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemNotificationServiceProtocol
import GemstonePrimitivesTestKit
import GemstoneServices

public extension InAppNotificationService {
    static func mock(
        service: any GemNotificationServiceProtocol = GemNotificationServiceMock(),
    ) -> Self {
        InAppNotificationService(service: service)
    }
}
