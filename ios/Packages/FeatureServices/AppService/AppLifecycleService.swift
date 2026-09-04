// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemPerpetualServiceProtocol
import protocol Gemstone.GemStreamSubscriptionServiceProtocol
import protocol Gemstone.GemTransactionStateServiceProtocol
import GemstonePrimitives
import WalletConnectorService
import ConnectionStatusService
import GemstoneServices
import protocol Gemstone.GemDeviceServiceProtocol
import Store
import protocol Gemstone.GemWalletSessionServiceProtocol
import Foundation
import Primitives
import StreamService
import SwiftUI

public actor AppLifecycleService: Sendable {
    private let walletConnector: any WalletConnectorServiceable
    private let connectionStatusObserver: ConnectionStatusObserver
    private let deviceService: any GemDeviceServiceProtocol
    private let subscriptionsObserver: SubscriptionsObserver
    private let streamObserverService: StreamObserverService
    private let streamSubscriptionService: any GemStreamSubscriptionServiceProtocol
    private let perpetualService: any GemPerpetualServiceProtocol
    private let perpetualObserver: any PerpetualObservable
    private let walletSessionService: any GemWalletSessionServiceProtocol
    private let transactionStateService: any GemTransactionStateServiceProtocol

    public init(
        walletConnector: any WalletConnectorServiceable,
        connectionStatusObserver: ConnectionStatusObserver,
        deviceService: any GemDeviceServiceProtocol,
        subscriptionsObserver: SubscriptionsObserver,
        streamObserverService: StreamObserverService,
        streamSubscriptionService: any GemStreamSubscriptionServiceProtocol,
        perpetualService: any GemPerpetualServiceProtocol,
        perpetualObserver: any PerpetualObservable,
        walletSessionService: any GemWalletSessionServiceProtocol,
        transactionStateService: any GemTransactionStateServiceProtocol,
    ) {
        self.walletConnector = walletConnector
        self.connectionStatusObserver = connectionStatusObserver
        self.deviceService = deviceService
        self.subscriptionsObserver = subscriptionsObserver
        self.streamObserverService = streamObserverService
        self.streamSubscriptionService = streamSubscriptionService
        self.perpetualService = perpetualService
        self.perpetualObserver = perpetualObserver
        self.walletSessionService = walletSessionService
        self.transactionStateService = transactionStateService
    }

    public func setup() async {
        async let walletConnect: () = setupWalletConnect()
        async let device: () = setupDeviceObserver()
        async let observers: () = connectObservers()

        _ = await (walletConnect, device, observers)
    }

    public func updateWalletConnections() async {
        async let assets: () = setupPriceAssets()
        async let perpetual: () = connectPerpetual()
        async let stream: () = connectStreamObserver()
        _ = await (assets, perpetual, stream)
    }

    public func updatePerpetualConnection() async {
        let wallet = await walletSessionService.currentWallet
        do {
            let connect = try await perpetualService.syncEnablement(wallet: wallet?.json(), trigger: .scheduled)
            await updatePerpetualObserver(wallet: wallet, connect: connect)
        } catch {
            debugLog("AppLifecycleService perpetual enablement error: \(error)")
        }
    }

    public func handleScenePhase(_ phase: ScenePhase) async {
        switch phase {
        case .active:
            debugLog("AppLifecycleService: App active — connecting observers")
            await connectObservers()
        case .background:
            debugLog("AppLifecycleService: App background — disconnecting observers")
            await disconnectObservers()
        case .inactive:
            debugLog("AppLifecycleService: App inactive")
        @unknown default:
            break
        }
    }
}

// MARK: - Private

extension AppLifecycleService {
    private func setupWalletConnect() async {
        do {
            try walletConnector.configure()
            if try await walletConnector.hasSessions() {
                await walletConnector.setup()
            }
        } catch {
            debugLog("AppLifecycleService setupWalletConnect error: \(error)")
        }
    }

    private func setupDeviceObserver() async {
        do {
            for try await _ in subscriptionsObserver.observe().dropFirst() {
                try await deviceService.synchronizeIfNeeded()
            }
        } catch {
            debugLog("AppLifecycleService setupDeviceObserver error: \(error)")
        }
    }

    private func setupPriceAssets() async {
        guard let walletId = walletSessionService.currentWalletId else { return }
        do {
            try await streamSubscriptionService.setupAssets(walletId: walletId.id)
        } catch {
            debugLog("AppLifecycleService setupPriceAssets error: \(error)")
        }
    }

    private func connectObservers() async {
        async let connection: () = connectionStatusObserver.start()
        async let stream: () = connectStreamObserver()
        async let perpetual: () = connectPerpetual()
        async let pending: () = trackPendingTransactions()
        _ = await (connection, stream, perpetual, pending)
    }

    private func trackPendingTransactions() async {
        do {
            try await transactionStateService.trackPending()
        } catch {
            debugLog("AppLifecycleService pending tracking error: \(error)")
        }
    }

    private func connectStreamObserver() async {
        guard walletSessionService.currentWalletId != nil else {
            await streamObserverService.disconnect()
            return
        }
        await registerDevice()
        await streamObserverService.connect()
    }

    private func registerDevice() async {
        do {
            try await deviceService.synchronizeIfNeeded()
        } catch {
            debugLog("AppLifecycleService registerDevice error: \(error)")
        }
    }

    private func connectPerpetual() async {
        let wallet = await walletSessionService.currentWallet
        let connect = perpetualService.shouldConnectPerpetuals(wallet: wallet?.json())
        await updatePerpetualObserver(wallet: wallet, connect: connect)
    }

    private func updatePerpetualObserver(wallet: Wallet?, connect: Bool) async {
        if connect, let wallet {
            await perpetualObserver.setup(for: wallet)
        } else {
            await perpetualObserver.disconnect()
        }
    }

    private func disconnectObservers() async {
        transactionStateService.stopTracking()
        async let connection: () = connectionStatusObserver.stop()
        async let price: () = streamObserverService.disconnect()
        async let perpetual: () = perpetualObserver.disconnect()
        _ = await (connection, price, perpetual)
    }
}
