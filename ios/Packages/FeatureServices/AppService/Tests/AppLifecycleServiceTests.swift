// Copyright (c). Gem Wallet. All rights reserved.

@testable import AppService
import GemstonePrimitivesTestKit
import AppServiceTestKit
import class Gemstone.GemWalletSessionService
import Foundation
import GemstoneServices
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
import Testing

struct AppLifecycleServiceTests {
    private let wallet = Wallet.mock(accounts: [.mock(chain: .hyperliquid)])

    @Test
    func updateWalletConnectionsConnectsWhenCoreSaysSo() async throws {
        let observer = PerpetualObserverMock()
        let service = try AppLifecycleService.mock(
            hyperliquidObserverService: observer,
            walletSessionService: GemWalletSessionService.mock(wallet: wallet),
        )

        await service.updateWalletConnections()

        #expect(await observer.isConnected == true)
    }

    @Test
    func updateWalletConnectionsSkipsWhenCoreDeclines() async throws {
        let observer = PerpetualObserverMock()
        let perpetuals = GemPerpetualServiceMock()
        let service = try AppLifecycleService.mock(
            hyperliquidObserverService: observer,
            perpetualService: perpetuals,
            walletSessionService: GemWalletSessionService.mock(wallet: wallet),
        )
        perpetuals.connects = false

        await service.updateWalletConnections()

        #expect(await observer.isConnected == false)
    }

    @Test
    func updateWalletConnectionsDisconnectsWhenCoreStopsConnecting() async throws {
        let observer = PerpetualObserverMock()
        let perpetuals = GemPerpetualServiceMock()
        let service = try AppLifecycleService.mock(
            hyperliquidObserverService: observer,
            perpetualService: perpetuals,
            walletSessionService: GemWalletSessionService.mock(wallet: wallet),
        )
        await service.updateWalletConnections()

        perpetuals.connects = false
        await service.updateWalletConnections()

        #expect(await observer.isConnected == false)
    }

    @Test
    func updateWalletConnectionsDisconnectsWhenNoCurrentWallet() async throws {
        let observer = PerpetualObserverMock()
        let session = try GemWalletSessionService.mock(wallet: wallet)
        let service = AppLifecycleService.mock(hyperliquidObserverService: observer, walletSessionService: session)
        await service.updateWalletConnections()

        try session.setCurrent(walletId: nil)
        await service.updateWalletConnections()

        #expect(await observer.isConnected == false)
    }

    @Test
    func updatePerpetualConnectionDisconnectsWhenDisabled() async throws {
        let observer = PerpetualObserverMock()
        let perpetuals = GemPerpetualServiceMock()
        let service = try AppLifecycleService.mock(
            hyperliquidObserverService: observer,
            perpetualService: perpetuals,
            walletSessionService: GemWalletSessionService.mock(wallet: wallet),
        )
        await service.updateWalletConnections()

        perpetuals.isPerpetualEnabled = false
        await service.updatePerpetualConnection()

        #expect(await observer.isConnected == false)
    }

    @Test
    func updatePerpetualConnectionUpdatesMarketsWhenEnabled() async throws {
        let perpetuals = GemPerpetualServiceMock()
        let service = AppLifecycleService.mock(perpetualService: perpetuals)

        await service.updatePerpetualConnection()

        #expect(perpetuals.syncMarketsCount == 1)
        #expect(perpetuals.clearMarketsCount == 0)
    }

    @Test
    func updatePerpetualConnectionClearsMarketsWhenDisabled() async throws {
        let perpetuals = GemPerpetualServiceMock()
        perpetuals.isPerpetualEnabled = false
        let service = AppLifecycleService.mock(perpetualService: perpetuals)

        await service.updatePerpetualConnection()

        #expect(perpetuals.clearMarketsCount == 1)
        #expect(try perpetuals.marketsUpdatedAt() == nil)
    }

    @Test
    func updateWalletConnectionsKeepsMarketsUntouched() async throws {
        let perpetuals = GemPerpetualServiceMock()
        let service = try AppLifecycleService.mock(
            perpetualService: perpetuals,
            walletSessionService: GemWalletSessionService.mock(wallet: wallet),
        )

        await service.updateWalletConnections()

        #expect(perpetuals.syncMarketsCount == 0)
        #expect(perpetuals.clearMarketsCount == 0)
    }
}
