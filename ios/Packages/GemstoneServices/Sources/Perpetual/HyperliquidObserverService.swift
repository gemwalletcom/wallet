// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import class Gemstone.GemPerpetualService
import protocol Gemstone.GemPerpetualServiceProtocol
import class Gemstone.GemPerpetualStreamService
import Foundation
import struct Gemstone.GemPerpetualConnection
import enum Gemstone.GemPerpetualSubscription
import Primitives
import WebSocketClient

public actor HyperliquidObserverService: PerpetualObservable {
    private let perpetualService: any GemPerpetualServiceProtocol
    private let webSocket: any WebSocketConnectable
    private let streamService: GemPerpetualStreamService

    private var observeTask: Task<Void, Never>?
    private var currentWallet: Wallet?

    public let chartService: any ChartStreamable

    public init(
        webSocketURL: URL,
        perpetualService: GemPerpetualService,
        reconnection: any Reconnectable,
    ) {
        let webSocket = WebSocketConnection(url: webSocketURL, reconnection: reconnection)
        self.webSocket = webSocket
        chartService = ChartObserverService()
        self.perpetualService = perpetualService
        streamService = GemPerpetualStreamService(
            perpetual: perpetualService,
            connection: PerpetualStreamConnection(webSocket: webSocket),
        )
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

        await streamService.disconnected()
        await webSocket.disconnect()
    }

    public func subscribe(_ subscription: GemPerpetualSubscription) async throws {
        try await streamService.subscribe(subscription: subscription)
    }

    public func unsubscribe(_ subscription: GemPerpetualSubscription) async throws {
        try await streamService.unsubscribe(subscription: subscription)
    }

    @discardableResult
    public func update(for wallet: Wallet) async -> PerpetualAccountMode? {
        guard let address = wallet.hyperliquidAccount?.address else { return nil }
        do {
            return try await perpetualService.syncPositions(walletId: wallet.id, address: address)
        } catch {
            debugLog("HyperliquidObserver: update failed: \(error)")
            return nil
        }
    }

    // MARK: - Private

    private func connect(for wallet: Wallet) async {
        guard currentWallet?.id != wallet.id else { return }

        await disconnect()
        currentWallet = wallet

        let connection: GemPerpetualConnection?
        do {
            connection = try await perpetualService.connection(wallet: wallet.json())
        } catch {
            debugLog("HyperliquidObserver: connection failed: \(error)")
            return
        }
        guard let connection, let mode = try? PerpetualAccountMode(connection.mode) else { return }

        guard observeTask == nil else { return }

        observeTask = Task { [weak self] in
            guard let self else { return }
            await observeConnection(walletId: wallet.id, address: connection.address, mode: mode)
        }
    }

    private func observeConnection(walletId: WalletId, address: String, mode: PerpetualAccountMode) async {
        for await event in await webSocket.connect() {
            guard !Task.isCancelled else { break }

            switch event {
            case .connected:
                await handleConnected(address: address, mode: mode)
            case let .message(data):
                await handle(data, walletId: walletId, mode: mode)
            case .disconnected:
                await streamService.disconnected()
            }
        }
    }

    private func handleConnected(address: String, mode: PerpetualAccountMode) async {
        do {
            try await streamService.connected(address: address, mode: mode.json())
        } catch {
            debugLog("HyperliquidObserver: subscribe failed: \(error)")
        }
    }

    private func handle(_ data: Data, walletId: WalletId, mode: PerpetualAccountMode) async {
        do {
            guard let candle = try await streamService.handle(walletId: walletId.id, mode: mode.json(), data: data) else { return }
            try await chartService.yield(Primitives.ChartCandleUpdate(candle))
        } catch {
            debugLog("HyperliquidObserver: handle message failed: \(error)")
        }
    }

}
