// Copyright (c). Gem Wallet. All rights reserved.

import BalanceService
import BalanceServiceTestKit
import Foundation
import NFTService
import NFTServiceTestKit
import PerpetualService
import PerpetualServiceTestKit
import Preferences
import PreferencesTestKit
import PriceAlertService
import PriceAlertServiceTestKit
import PriceService
import PriceServiceTestKit
import Store
import StoreTestKit
import StreamService
import TransactionsService
import TransactionsServiceTestKit

public extension StreamEventService {
    static func mock(
        walletStore: WalletStore = .mock(),
        notificationStore: InAppNotificationStore = .mock(),
        priceService: PriceService = .mock(),
        priceAlertService: PriceAlertService = .mock(),
        balanceUpdater: any BalanceUpdater = .mock(),
        transactionsService: TransactionsService = .mock(),
        nftService: NFTService = .mock(),
        perpetualService: any HyperliquidPerpetualServiceable = PerpetualServiceMock(),
        preferences: Preferences = .mock(),
    ) -> StreamEventService {
        StreamEventService(
            walletStore: walletStore,
            notificationStore: notificationStore,
            priceService: priceService,
            priceAlertService: priceAlertService,
            balanceUpdater: balanceUpdater,
            transactionsService: transactionsService,
            nftService: nftService,
            perpetualService: perpetualService,
            preferences: preferences,
        )
    }
}
