// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemAmountService
import class Gemstone.GemNameService
import class Gemstone.GemPreferencesService
import class Gemstone.GemStaticApiClient
import class Gemstone.GemWalletSessionService
import GemstonePrimitivesTestKit
import GemstoneServices
import NativeProviderService
import StoreTestKit

public extension GemAmountService {
    static func mock(session: GemWalletSessionService = .mock()) -> GemAmountService {
        let preferences = GemPreferencesService(store: GemPreferencesStoreMock())
        return GemAmountService(
            stake: GatewayService.mock().stakeService(
                staticApi: GemStaticApiClient(provider: NativeProvider()),
                store: GemstoneStakeStore(store: .mock()),
                names: .mock(),
                explorer: .mock(),
                preferences: preferences,
                session: .mock(),
            ),
            preferences: preferences,
            session: session,
        )
    }
}
