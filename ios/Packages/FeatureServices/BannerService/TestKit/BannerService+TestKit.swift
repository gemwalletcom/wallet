// Copyright (c). Gem Wallet. All rights reserved.

import BannerService
import NotificationService
import NotificationServiceTestKit
import Store
import StoreTestKit

public extension BannerService {
    static func mock(
        store: BannerStore = .mock(),
        pushNotificationService: PushNotificationEnablerService = .mock(),
    ) -> Self {
        BannerService(
            store: store,
            pushNotificationService: pushNotificationService,
        )
    }
}
