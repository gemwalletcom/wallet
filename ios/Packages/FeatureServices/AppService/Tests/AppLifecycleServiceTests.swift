// Copyright (c). Gem Wallet. All rights reserved.

@testable import AppService
import AppServiceTestKit
import Foundation
import class Gemstone.GemWalletSessionService
import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct AppLifecycleServiceTests {
    private let wallet = Wallet.mock(accounts: [.mock(chain: .hyperliquid)])

    @Test(.timeLimit(.minutes(1)))
    func aFailedDeviceSyncDoesNotEndTheAccountObservation() async {
        let db = DB.mock(wallets: [.mock(id: .mock(address: "first"), accounts: [.mock(chain: .ethereum)])])
        let synchronized = AsyncStream<Void>.makeStream()
        let device = GemDeviceServiceMock(syncError: AnyError("offline"), onSynchronize: { synchronized.continuation.yield(()) })
        let service = AppLifecycleService.mock(deviceService: device, subscriptionsObserver: SubscriptionsObserver(dbQueue: db.dbQueue))
        let running = Task { await service.setup() }
        let store = WalletStore(db: db)
        let changes = Task {
            for index in 0 ... .max where !Task.isCancelled {
                try? store.addWallet(.mock(id: .mock(address: "wallet-\(index)"), accounts: [.mock(chain: .ethereum, address: "wallet-\(index)")]))
                try? await Task.sleep(for: .milliseconds(20))
            }
        }

        var synchronizations = synchronized.stream.makeAsyncIterator()
        await synchronizations.next()
        await synchronizations.next()
        changes.cancel()
        running.cancel()

        #expect(await device.synchronizeIfNeededCalls >= 2)
    }

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

        try session.setCurrentWalletId(walletId: nil)
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
    func updatePerpetualConnectionUpdatesMarketsWhenEnabled() async {
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
    func updateWalletConnectionsRefreshesStaleMarkets() async throws {
        let perpetuals = GemPerpetualServiceMock()
        let service = try AppLifecycleService.mock(
            perpetualService: perpetuals,
            walletSessionService: GemWalletSessionService.mock(wallet: wallet),
        )

        await service.updateWalletConnections()

        #expect(perpetuals.syncMarketsCount == 1, "a wallet switch refreshes markets that were never synced")
        #expect(perpetuals.clearMarketsCount == 0)
    }
}
