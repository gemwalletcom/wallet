// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemNameService
import class Gemstone.GemPreferencesService
import class Gemstone.GemStakeService
import class Gemstone.GemStaticApiClient
import GemstonePrimitivesTestKit
import GemstoneServices
import NativeProviderService
import StoreTestKit

public extension GemStakeService {
    static func mock() -> GemStakeService {
        GatewayService.mock().stakeService(
            staticApi: GemStaticApiClient(provider: NativeProvider()),
            store: GemstoneStakeStore(store: .mock()),
            names: .mock(),
            explorer: .mock(),
            preferences: GemPreferencesService(store: GemPreferencesStoreMock()),
            session: .mock(),
        )
    }
}
