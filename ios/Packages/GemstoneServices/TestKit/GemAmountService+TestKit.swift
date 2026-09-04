// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemAmountService
import class Gemstone.GemPreferencesService
import class Gemstone.GemStaticApiClient
import GemstonePrimitivesTestKit
import GemstoneServices
import NativeProviderService
import Store
import StoreTestKit

public extension GemAmountService {
    static func mock() -> GemAmountService {
        let preferences = GemPreferencesService(store: GemPreferencesStoreMock())
        return GemAmountService(
            stake: GatewayService.mock().stakeService(
                staticApi: GemStaticApiClient(provider: NativeProvider()),
                store: GemstoneStakeStore(store: .mock()),
                addressStore: GemstoneAddressStore(store: .mock()),
                explorer: .mock(),
                preferences: preferences,
                session: .mock(),
            ),
            preferences: preferences,
            session: .mock(),
        )
    }
}
