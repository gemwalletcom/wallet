// Copyright (c). Gem Wallet. All rights reserved.

import BannerService
import Foundation
import NotificationService
import NotificationServiceTestKit
import Preferences
import PreferencesTestKit
import Store
import StoreTestKit

public extension BannerSetupService {
    static func mock(
        store: BannerStore = .mock(),
        preferences: Preferences = .mock(),
    ) -> Self {
        BannerSetupService(
            store: store,
            preferences: preferences,
        )
    }
}
