// Copyright (c). Gem Wallet. All rights reserved.

import ConnectionsService
import ConnectionStatusService
import GemstoneServices
import Foundation
import PerpetualService
import Preferences
import Primitives
import StreamService
import SwiftUI
import WalletSessionService

public actor AppLifecycleService: Sendable {
    private let preferences: Preferences
    private let connectionsService: ConnectionsService
    private let connectionStatusObserver: ConnectionStatusObserver
    private let deviceObserverService: DeviceObserverService
    private let streamObserverService: StreamObserverService
    private let streamSubscriptionService: StreamSubscriptionService
    private let perpetualEnablerService: PerpetualEnablerService
    private let walletSessionService: any WalletSessionManageable

    public init(
        preferences: Preferences,
        connectionsService: ConnectionsService,
        connectionStatusObserver: ConnectionStatusObserver,
        deviceObserverService: DeviceObserverService,
        streamObserverService: StreamObserverService,
        streamSubscriptionService: StreamSubscriptionService,
        perpetualEnablerService: PerpetualEnablerService,
        walletSessionService: any WalletSessionManageable,
    ) {
        self.preferences = preferences
        self.connectionsService = connectionsService
        self.connectionStatusObserver = connectionStatusObserver
        self.deviceObserverService = deviceObserverService
        self.streamObserverService = streamObserverService
        self.streamSubscriptionService = streamSubscriptionService
        self.perpetualEnablerService = perpetualEnablerService
        self.walletSessionService = walletSessionService
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
        await perpetualEnablerService.updateEnablement(wallet: walletSessionService.currentWallet)
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
            try await connectionsService.setup()
        } catch {
            debugLog("AppLifecycleService setupWalletConnect error: \(error)")
        }
    }

    private func setupDeviceObserver() async {
        do {
            try await deviceObserverService.startSubscriptionsObserver()
        } catch {
            debugLog("AppLifecycleService setupDeviceObserver error: \(error)")
        }
    }

    private func setupPriceAssets() async {
        do {
            try await streamSubscriptionService.setupAssets()
        } catch {
            debugLog("AppLifecycleService setupPriceAssets error: \(error)")
        }
    }

    private func connectObservers() async {
        async let connection: () = connectionStatusObserver.start()
        async let stream: () = connectStreamObserver()
        async let perpetual: () = connectPerpetual()
        _ = await (connection, stream, perpetual)
    }

    private func connectStreamObserver() async {
        guard walletSessionService.currentWalletId != nil else {
            await streamObserverService.disconnect()
            return
        }
        if preferences.isDeviceRegistered == false {
            await registerDevice()
        }
        await streamObserverService.connect()
    }

    private func registerDevice() async {
        do {
            try await deviceObserverService.synchronizeIfNeeded()
        } catch {
            debugLog("AppLifecycleService registerDevice error: \(error)")
        }
    }

    private func connectPerpetual() async {
        await perpetualEnablerService.updateConnection(wallet: walletSessionService.currentWallet)
    }

    private func disconnectObservers() async {
        async let connection: () = connectionStatusObserver.stop()
        async let price: () = streamObserverService.disconnect()
        async let perpetual: () = perpetualEnablerService.disconnect()
        _ = await (connection, price, perpetual)
    }
}
