// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemApiClient
import class Gemstone.GemAssetsService
import class Gemstone.GemPaymentService
import class Gemstone.GemPreferencesService
import GemstonePrimitivesTestKit
import GemstoneServices
import Store
import StoreTestKit

public extension GemAssetsService {
    static func mock(db: DB = .mock()) -> GemAssetsService {
        GatewayService.mock().assetsService(
            api: GemApiClient(provider: StubAlienProvider()),
            store: GemstoneAssetStore(assetStore: .mock(db: db), balanceStore: .mock(db: db)),
            price: .mock(db: db),
            preferences: GemPreferencesService(store: GemPreferencesStoreMock()),
            session: .mock(),
        )
    }
}

public extension GemPaymentService {
    static func mock() -> GemPaymentService {
        GemPaymentService(provider: StubAlienProvider(), assets: .mock())
    }
}
