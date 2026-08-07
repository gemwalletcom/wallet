// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import GemAPITestKit
import NotificationService
import Store
import StoreTestKit

public extension InAppNotificationService {
    static func mock(
        apiService: GemAPINotificationService = GemAPINotificationServiceMock(),
        store: InAppNotificationStore = .mock(),
    ) -> Self {
        InAppNotificationService(
            apiService: apiService,
            store: store,
        )
    }
}
