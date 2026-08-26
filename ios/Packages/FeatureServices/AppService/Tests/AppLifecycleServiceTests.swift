// Copyright (c). Gem Wallet. All rights reserved.

@testable import AppService
import Foundation
import GemstoneServices
import GemstoneServicesTestKit
import Preferences
import PreferencesTestKit
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing
import WalletSessionService
import WalletSessionServiceTestKit

struct AppLifecycleServiceTests {
    @Test
    func updateWalletConnectionsConnectsHyperliquidForMultiCoinWallet() async throws {
        let (service, observer, _, _) = try makeService(perpetualEnabled: true, wallets: [.hyperliquid], current: .hyperliquid)

        await service.updateWalletConnections()

        #expect(await observer.isConnected == true)
    }

    @Test
    func updateWalletConnectionsSkipsHyperliquidForSingleChainWallet() async throws {
        let (service, observer, _, _) = try makeService(perpetualEnabled: true, wallets: [.single], current: .single)

        await service.updateWalletConnections()

        #expect(await observer.isConnected == false)
    }

    @Test
    func updateWalletConnectionsDisconnectsWhenSwitchingToSingleChainWallet() async throws {
        let (service, observer, _, session) = try makeService(perpetualEnabled: true, wallets: [.hyperliquid, .single], current: .hyperliquid)
        await service.updateWalletConnections()

        session.setCurrent(walletId: Wallet.single.id)
        await service.updateWalletConnections()

        #expect(await observer.isConnected == false)
    }

    @Test
    func updateWalletConnectionsDisconnectsWhenNoCurrentWallet() async throws {
        let (service, observer, _, session) = try makeService(perpetualEnabled: true, wallets: [.hyperliquid], current: .hyperliquid)
        await service.updateWalletConnections()

        session.setCurrent(walletId: nil)
        await service.updateWalletConnections()

        #expect(await observer.isConnected == false)
    }

    @Test
    func updateWalletConnectionsSkipsHyperliquidWhenDisabled() async throws {
        let (service, observer, _, _) = try makeService(perpetualEnabled: false, wallets: [.hyperliquid], current: .hyperliquid)

        await service.updateWalletConnections()

        #expect(await observer.isConnected == false)
    }

    @Test
    func updatePerpetualConnectionDisconnectsWhenDisabled() async throws {
        let (service, observer, preferences, _) = try makeService(perpetualEnabled: true, wallets: [.hyperliquid], current: .hyperliquid)
        await service.updateWalletConnections()

        preferences.isPerpetualEnabled = false
        await service.updatePerpetualConnection()

        #expect(await observer.isConnected == false)
    }

    @Test
    func updatePerpetualConnectionSkipsForSingleChainWallet() async throws {
        let (service, observer, _, _) = try makeService(perpetualEnabled: true, wallets: [.single], current: .single)
        await service.updateWalletConnections()

        await service.updatePerpetualConnection()

        #expect(await observer.isConnected == false)
    }

    @Test
    func updatePerpetualConnectionUpdatesMarketsWhenEnabled() async throws {
        let (service, _, preferences, _) = try makeService(perpetualEnabled: true)

        await service.updatePerpetualConnection()

        #expect(preferences.perpetualMarketsUpdatedAt != nil)
    }

    @Test
    func updatePerpetualConnectionClearsMarketsWhenDisabled() async throws {
        let (service, _, preferences, _) = try makeService(perpetualEnabled: false)
        preferences.perpetualMarketsUpdatedAt = .now

        await service.updatePerpetualConnection()

        #expect(preferences.perpetualMarketsUpdatedAt == nil)
    }

    @Test
    func updateWalletConnectionsKeepsMarketsUntouched() async throws {
        let (service, _, preferences, _) = try makeService(perpetualEnabled: true, wallets: [.hyperliquid], current: .hyperliquid)
        let updatedAt = Date(timeIntervalSince1970: 0)
        preferences.perpetualMarketsUpdatedAt = updatedAt

        await service.updateWalletConnections()

        #expect(preferences.perpetualMarketsUpdatedAt == updatedAt)
    }
}

extension AppLifecycleServiceTests {
    func makeService(
        perpetualEnabled: Bool,
        wallets: [Wallet] = [],
        current: Wallet? = nil,
    ) throws -> (AppLifecycleService, PerpetualObserverMock, Preferences, any WalletSessionManageable) {
        let preferences = Preferences.mock()
        preferences.isPerpetualEnabled = perpetualEnabled
        let observer = PerpetualObserverMock()
        let store = WalletStore.mock(db: .mockWithChains([.hyperliquid]))
        for wallet in wallets {
            try store.addWallet(wallet)
        }
        let walletSessionService = WalletSessionService.mock(
            store: store,
            preferences: ObservablePreferences(preferences: preferences),
        )
        if let current {
            walletSessionService.setCurrent(walletId: current.id)
        }
        let service = try AppLifecycleService.mock(
            preferences: preferences,
            hyperliquidObserverService: observer,
            perpetualService: PerpetualService.mock(db: DB.mockPerpetualAssets(), preferences: preferences),
            walletSessionService: walletSessionService,
        )
        return (service, observer, preferences, walletSessionService)
    }
}

private extension Wallet {
    static let hyperliquid = Wallet.mock(type: .multicoin, accounts: [.mock(chain: .hyperliquid)])
    static let single = Wallet.mock(id: .single(chain: .bitcoin, address: "bc1"), type: .single)
}
