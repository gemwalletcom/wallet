// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemPerpetualConnection
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
    }

    deinit {
        observeTask?.cancel()
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

    public func subscribe(_ subscription: GemPerpetualSubscription) async throws {
        try await streamService.subscribe(subscription: subscription)
    }

    public func unsubscribe(_ subscription: GemPerpetualSubscription) async throws {
        try await streamService.unsubscribe(subscription: subscription)
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
        let connection: GemPerpetualConnection?
        do {
            connection = try await perpetualService.connection(wallet: wallet.toGem())
        } catch {
            debugLog("HyperliquidObserver: connection failed: \(error)")
            connection = nil
        }
        guard !Task.isCancelled else { return }
        guard let connection else {
            await closeConnection()
            return
        }
        let mode = connection.mode.toPrimitives()

        for await event in events {
            guard !Task.isCancelled else { break }

            switch event {
            case .connected:
                await onConnected(address: connection.address, mode: mode)
            case let .message(data):
                await onMessage(data, walletId: wallet.id, mode: mode)
            case .disconnected:
                await streamService.disconnected()
            }
        }
    }

    private func onConnected(address: String, mode: PerpetualAccountMode) async {
        do {
            try await streamService.connected(address: address, mode: mode.toGem())
        } catch {
            debugLog("HyperliquidObserver: subscribe failed: \(error)")
        }
    }

    private func onMessage(_ data: Data, walletId: WalletId, mode: PerpetualAccountMode) async {
        do {
            guard let candle = try await streamService.candleUpdate(walletId: walletId, mode: mode.toGem(), data: data) else { return }
            await chartService.yield(candle.toPrimitives())
        } catch {
            debugLog("HyperliquidObserver: handle message failed: \(error)")
        }
    }
}
