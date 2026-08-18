// Copyright (c). Gem Wallet. All rights reserved.

@testable import AppService
import Foundation
import PerpetualService
import PerpetualServiceTestKit
import Preferences
import PreferencesTestKit
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct AppLifecycleServiceTests {
    @Test
    func setupWalletConnectsHyperliquidForMultiCoinWallet() async throws {
        let (service, observer, _) = try makeService(perpetualEnabled: true)

        await service.setupWallet(.mock(type: .multicoin, accounts: [.mock(chain: .hyperliquid)]))

        #expect(await observer.isConnected == true)
    }

    @Test
    func setupWalletSkipsHyperliquidForSingleChainWallet() async throws {
        let (service, observer, _) = try makeService(perpetualEnabled: true)

        await service.setupWallet(.mock(type: .single))

        #expect(await observer.isConnected == false)
    }

    @Test
    func setupWalletDisconnectsWhenSwitchingToSingleChainWallet() async throws {
        let (service, observer, _) = try makeService(perpetualEnabled: true)
        await service.setupWallet(.mock(type: .multicoin, accounts: [.mock(chain: .hyperliquid)]))

        await service.setupWallet(.mock(type: .single))

        #expect(await observer.isConnected == false)
    }

    @Test
    func setupWalletSkipsHyperliquidWhenDisabled() async throws {
        let (service, observer, _) = try makeService(perpetualEnabled: false)

        await service.setupWallet(.mock(type: .multicoin, accounts: [.mock(chain: .hyperliquid)]))

        #expect(await observer.isConnected == false)
    }

    @Test
    func updatePerpetualConnectionDisconnectsWhenDisabled() async throws {
        let (service, observer, preferences) = try makeService(perpetualEnabled: true)
        await service.setupWallet(.mock(type: .multicoin, accounts: [.mock(chain: .hyperliquid)]))

        preferences.isPerpetualEnabled = false
        await service.updatePerpetualConnection()

        #expect(await observer.isConnected == false)
    }

    @Test
    func updatePerpetualConnectionSkipsForSingleChainWallet() async throws {
        let (service, observer, _) = try makeService(perpetualEnabled: true)
        await service.setupWallet(.mock(type: .single))

        await service.updatePerpetualConnection()

        #expect(await observer.isConnected == false)
    }

    @Test
    func updatePerpetualConnectionUpdatesMarketsWhenEnabled() async throws {
        let (service, _, preferences) = try makeService(perpetualEnabled: true)

        await service.updatePerpetualConnection()

        #expect(preferences.perpetualMarketsUpdatedAt != nil)
    }

    @Test
    func updatePerpetualConnectionClearsMarketsWhenDisabled() async throws {
        let (service, _, preferences) = try makeService(perpetualEnabled: false)
        preferences.perpetualMarketsUpdatedAt = .now

        await service.updatePerpetualConnection()

        #expect(preferences.perpetualMarketsUpdatedAt == nil)
    }

    @Test
    func setupWalletKeepsMarketsUntouched() async throws {
        let (service, _, preferences) = try makeService(perpetualEnabled: true)
        let updatedAt = Date(timeIntervalSince1970: 0)
        preferences.perpetualMarketsUpdatedAt = updatedAt

        await service.setupWallet(.mock(type: .multicoin, accounts: [.mock(chain: .hyperliquid)]))

        #expect(preferences.perpetualMarketsUpdatedAt == updatedAt)
    }
}

extension AppLifecycleServiceTests {
    func makeService(perpetualEnabled: Bool) throws -> (AppLifecycleService, PerpetualObserverMock, Preferences) {
        let preferences = Preferences.mock()
        preferences.isPerpetualEnabled = perpetualEnabled
        let observer = PerpetualObserverMock()
        let service = try AppLifecycleService.mock(
            preferences: preferences,
            hyperliquidObserverService: observer,
            perpetualService: PerpetualService.mock(db: DB.mockPerpetualAssets(), preferences: preferences),
        )
        return (service, observer, preferences)
    }
}
