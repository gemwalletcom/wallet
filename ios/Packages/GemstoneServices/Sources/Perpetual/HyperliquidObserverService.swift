// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemPerpetualServiceProtocol
import protocol Gemstone.GemPerpetualStreamServiceProtocol
import enum Gemstone.GemPerpetualSubscription
import GemstonePrimitives
import Primitives
import WebSocketClient

public actor HyperliquidObserverService: PerpetualObservable {
    private let perpetualService: any GemPerpetualServiceProtocol
    private let webSocket: any WebSocketConnectable
    private let streamService: any GemPerpetualStreamServiceProtocol
    private let subscriptionChanges: AsyncStream<SubscriptionChange>.Continuation
    private let subscriptionTask: Task<Void, Never>

    private var observeTask: Task<Void, Never>?
    private var currentWallet: Wallet?
    private var pendingWalletId: WalletId?
    private var generation = 0

    public let chartService: ChartObserverService

    public init(
        webSocket: any WebSocketConnectable,
        perpetualService: any GemPerpetualServiceProtocol,
        streamService: any GemPerpetualStreamServiceProtocol,
        chartService: ChartObserverService = ChartObserverService(),
    ) {
        self.webSocket = webSocket
        self.perpetualService = perpetualService
        self.streamService = streamService
        self.chartService = chartService
        let (changes, continuation) = AsyncStream.makeStream(of: SubscriptionChange.self)
        subscriptionChanges = continuation
        subscriptionTask = Task {
            for await change in changes {
                await change.send(to: streamService)
            }
        }
    }

    deinit {
        observeTask?.cancel()
        subscriptionChanges.finish()
        subscriptionTask.cancel()
    }

    // MARK: - Public API

    public func setup(for wallet: Wallet) async {
        await connect(for: wallet)
    }

    public func disconnect() async {
        generation += 1
        pendingWalletId = nil
        await closeConnection()
    }

    public nonisolated func subscribe(_ subscription: GemPerpetualSubscription) {
        subscriptionChanges.yield(.subscribe(subscription))
    }

    public nonisolated func unsubscribe(_ subscription: GemPerpetualSubscription) {
        subscriptionChanges.yield(.unsubscribe(subscription))
    }

    // MARK: - Private

    private func connect(for wallet: Wallet) async {
        guard currentWallet?.id != wallet.id, pendingWalletId != wallet.id else { return }
        generation += 1
        let token = generation
        pendingWalletId = wallet.id

        await closeConnection()
        guard token == generation else { return }
        pendingWalletId = nil

        currentWallet = wallet
        observeTask = Task { [weak self] in
            guard let self else { return }
            await observeConnection(wallet: wallet)
        }
    }

    private func closeConnection() async {
        guard observeTask != nil else { return }

        observeTask?.cancel()
        observeTask = nil
        currentWallet = nil

        await streamService.disconnected()
        await webSocket.disconnect()
    }

    private func observeConnection(wallet: Wallet) async {
        let events = await webSocket.connect()
        let address: String?
        do {
            address = try await perpetualService.connectionAddress(wallet: wallet.toGem())
        } catch {
            debugLog("HyperliquidObserver: connection failed: \(error)")
            address = nil
        }
        guard !Task.isCancelled else { return }
        guard let address else {
            await closeConnection()
            return
        }

        for await event in events {
            guard !Task.isCancelled else { break }

            switch event {
            case .connected:
                await onConnected(address: address)
            case let .message(data):
                await onMessage(data, walletId: wallet.id)
            case .disconnected:
                await streamService.disconnected()
            }
        }
    }

    private func onConnected(address: String) async {
        do {
            try await streamService.connected(address: address)
        } catch {
            debugLog("HyperliquidObserver: subscribe failed: \(error)")
        }
    }

    private func onMessage(_ data: Data, walletId: WalletId) async {
        do {
            guard let candle = try await streamService.candleUpdate(walletId: walletId, data: data) else { return }
            await chartService.yield(candle.toPrimitives())
        } catch {
            debugLog("HyperliquidObserver: handle message failed: \(error)")
        }
    }
}

private enum SubscriptionChange {
    case subscribe(GemPerpetualSubscription)
    case unsubscribe(GemPerpetualSubscription)

    func send(to streamService: any GemPerpetualStreamServiceProtocol) async {
        do {
            switch self {
            case let .subscribe(subscription): try await streamService.subscribe(subscription: subscription)
            case let .unsubscribe(subscription): try await streamService.unsubscribe(subscription: subscription)
            }
        } catch {
            debugLog("HyperliquidObserver: \(self) failed: \(error)")
        }
    }
}
