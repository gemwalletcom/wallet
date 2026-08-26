// Copyright (c). Gem Wallet. All rights reserved.

import BannerService
import class Gemstone.GemBannerService
import GemstoneStore
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
            service: GemBannerService(store: GemstoneBannerStore(store: store)),
            pushNotificationService: pushNotificationService,
        )
    }
}
