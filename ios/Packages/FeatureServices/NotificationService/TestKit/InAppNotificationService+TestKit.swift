// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import GemAPITestKit
import NotificationService
import Store
import StoreTestKit
import WalletService
import WalletServiceTestKit

public extension InAppNotificationService {
    static func mock(
        apiService: GemAPINotificationService = GemAPINotificationServiceMock(),
        walletService: WalletService = .mock(),
        store: InAppNotificationStore = .mock(),
    ) -> Self {
        InAppNotificationService(
            apiService: apiService,
            walletService: walletService,
            store: store,
        )
    }
}
