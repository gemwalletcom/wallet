// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemNotificationServiceProtocol
import GemstonePrimitivesTestKit
import NotificationService
import Store
import StoreTestKit

public extension InAppNotificationService {
    static func mock(
        apiService: any GemNotificationServiceProtocol = GemNotificationServiceMock(),
        store: InAppNotificationStore = .mock(),
    ) -> Self {
        InAppNotificationService(
            apiService: apiService,
            store: store,
        )
    }
}
