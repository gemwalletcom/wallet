// Copyright (c). Gem Wallet. All rights reserved.

import ConnectionsService
import ConnectionStatusService
import DeviceService
import Foundation
import PerpetualService
import Preferences
import Primitives
import StreamService
import SwiftUI

public actor AppLifecycleService: Sendable {
    private let preferences: Preferences
    private let connectionsService: ConnectionsService
    private let connectionStatusObserver: ConnectionStatusObserver
    private let deviceObserverService: DeviceObserverService
    private let streamObserverService: StreamObserverService
    private let streamSubscriptionService: StreamSubscriptionService
    private let perpetualEnablerService: PerpetualEnablerService

    private var currentWallet: Wallet?

    public init(
        preferences: Preferences,
        connectionsService: ConnectionsService,
        connectionStatusObserver: ConnectionStatusObserver,
        deviceObserverService: DeviceObserverService,
        streamObserverService: StreamObserverService,
        streamSubscriptionService: StreamSubscriptionService,
        perpetualEnablerService: PerpetualEnablerService,
    ) {
        self.preferences = preferences
        self.connectionsService = connectionsService
        self.connectionStatusObserver = connectionStatusObserver
        self.deviceObserverService = deviceObserverService
        self.streamObserverService = streamObserverService
        self.streamSubscriptionService = streamSubscriptionService
        self.perpetualEnablerService = perpetualEnablerService
    }

    public func setup() async {
        async let walletConnect: () = setupWalletConnect()
        async let device: () = setupDeviceObserver()
        async let observers: () = connectObservers()

        _ = await (walletConnect, device, observers)
    }

    public func setupWallet(_ wallet: Wallet) async {
        currentWallet = wallet
        async let assets: () = setupPriceAssets(wallet: wallet)
        async let perpetual: () = connectPerpetual()
        async let stream: () = connectStreamObserver()
        _ = await (assets, perpetual, stream)
    }

    public func updatePerpetualConnection() async {
        await perpetualEnablerService.updateEnablement(wallet: currentWallet)
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

    private func setupPriceAssets(wallet: Wallet) async {
        do {
            try await streamSubscriptionService.setupAssets(walletId: wallet.id)
        } catch {
            debugLog("AppLifecycleService setupPriceAssets error: \(error)")
        }
    }

    private func connectObservers() async {
        async let connection: () = connectionStatusObserver.start()
        async let stream: () = connectStreamObserver()
        async let perpetual: () = connectPerpetual()
        async let nodeAuthToken: () = deviceObserverService.startNodeAuthTokenUpdates()
        _ = await (connection, stream, perpetual, nodeAuthToken)
    }

    private func connectStreamObserver() async {
        guard currentWallet != nil else { return }
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
        await perpetualEnablerService.updateConnection(wallet: currentWallet)
    }

    private func disconnectObservers() async {
        async let connection: () = connectionStatusObserver.stop()
        async let price: () = streamObserverService.disconnect()
        async let perpetual: () = perpetualEnablerService.disconnect()
        async let nodeAuthToken: () = deviceObserverService.stopNodeAuthTokenUpdates()
        _ = await (connection, price, perpetual, nodeAuthToken)
    }
}
