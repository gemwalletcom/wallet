// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemPerpetualSubscription
import Primitives
import WebSocketClient

public actor HyperliquidObserverService: PerpetualObservable {
    private let perpetualService: HyperliquidPerpetualServiceable
    private let webSocket: any WebSocketConnectable
    private let subscriptionService: HyperliquidSubscriptionService
    private let eventHandler: HyperliquidEventHandler

    private var observeTask: Task<Void, Never>?
    private var currentWallet: Wallet?

    public let chartService: any ChartStreamable

    public init(
        nodeProvider: any NodeURLFetchable,
        perpetualService: HyperliquidPerpetualServiceable,
    ) {
        let webSocket = WebSocketConnection(url: nodeProvider.node(for: .hyperCore))
        let chartService = ChartObserverService()

        self.webSocket = webSocket
        self.perpetualService = perpetualService
        self.chartService = chartService
        subscriptionService = HyperliquidSubscriptionService(webSocket: webSocket)
        eventHandler = HyperliquidEventHandler(perpetualService: perpetualService, chartService: chartService)
    }

    deinit {
        observeTask?.cancel()
    }

    // MARK: - Public API

    public func setup(for wallet: Wallet) async {
        await connect(for: wallet)
    }

    public func disconnect() async {
        guard observeTask != nil else { return }

        observeTask?.cancel()
        observeTask = nil
        currentWallet = nil

        await webSocket.disconnect()
    }

    public func subscribe(_ subscription: GemPerpetualSubscription) async throws {
        try await subscriptionService.subscribe(subscription)
    }

    public func unsubscribe(_ subscription: GemPerpetualSubscription) async throws {
        try await subscriptionService.unsubscribe(subscription)
    }

    public func update(for wallet: Wallet) async {
        guard let address = wallet.hyperliquidAccount?.address else { return }
        do {
            try await perpetualService.getPositions(walletId: wallet.id, address: address)
        } catch {
            debugLog("HyperliquidObserver: update failed: \(error)")
        }
    }

    // MARK: - Private

    private func connect(for wallet: Wallet) async {
        guard currentWallet?.id != wallet.id else { return }

        await disconnect()
        currentWallet = wallet
        let mode = await accountMode(for: wallet)
        await update(for: wallet)

        guard observeTask == nil else { return }

        observeTask = Task { [weak self] in
            guard let self else { return }
            await observeConnection(walletId: wallet.id, mode: mode)
        }
    }

    private func observeConnection(walletId: WalletId, mode: PerpetualAccountMode) async {
        for await event in await webSocket.connect() {
            guard !Task.isCancelled else { break }

            switch event {
            case .connected:
                await handleConnected(mode: mode)
            case let .message(data):
                await eventHandler.handle(data, walletId: walletId, mode: mode)
            case .disconnected:
                break
            }
        }
    }

    private func handleConnected(mode: PerpetualAccountMode) async {
        guard let address = currentWallet?.hyperliquidAccount?.address else { return }
        do {
            try await subscriptionService.connected(address: address, mode: mode)
        } catch {
            debugLog("HyperliquidObserver: subscribe failed: \(error)")
        }
    }

    private func accountMode(for wallet: Wallet) async -> PerpetualAccountMode {
        guard let address = wallet.hyperliquidAccount?.address else { return .standard }
        return await perpetualService.accountMode(walletId: wallet.id, address: address)
    }
}
