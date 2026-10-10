// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemBalanceService
import class Gemstone.GemDeviceApiClient
import class Gemstone.GemDeviceKeyService
import class Gemstone.GemDeviceService
import class Gemstone.GemPreferencesService
import class Gemstone.GemPriceAlertService
import class Gemstone.GemStreamSubscriptionService
import class Gemstone.GemSubscriptionService
import class Gemstone.GemWalletSessionService
import GemstonePrimitivesTestKit
import GemstoneServices
import Store
import StoreTestKit

public extension GemBalanceService {
    static func mock(db: DB = .mock()) -> GemBalanceService {
        let api = GemDeviceApiClient(provider: StubAlienProvider(), deviceKey: GemDeviceKeyService(store: GemSecureStoreMock()))
        let preferences = GemPreferencesService(store: GemPreferencesStoreMock())
        let session = GemWalletSessionService.mock(store: .mock(db: db))
        let balances = GemstoneBalanceStore(store: .mock(db: db))
        let device = GemDeviceService(
            api: api,
            subscriptions: GemSubscriptionService(api: api, session: session),
            platform: GemDevicePlatformMock(),
            preferences: preferences,
        )
        let alerts = GemPriceAlertService(
            api: api,
            preferences: preferences,
            store: GemstonePriceAlertStore(store: PriceAlertStore(db: db)),
            assets: .mock(),
            device: device,
            permissions: GemNotificationPermissionsMock(),
        )
        return GatewayService.mock().balanceService(
            store: balances,
            assets: .mock(),
            session: session,
            stream: GemStreamSubscriptionService(balances: balances, alerts: alerts, connection: GemStreamConnectionMock()),
        )
    }
}
