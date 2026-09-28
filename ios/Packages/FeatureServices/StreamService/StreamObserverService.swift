// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemStreamServiceProtocol
import GemstonePrimitives
import Primitives
import WebSocketClient

public actor StreamObserverService: Sendable {
    private let service: any GemStreamServiceProtocol
    private let webSocket: any WebSocketConnectable
    private let health: ConnectionComponentHealth
    private let reconnection: any Reconnectable
    private var observeTask: Task<Void, Never>?
    private var isActive = false
    private var failedAttempts: UInt32 = 0
    private var connectedAt: ContinuousClock.Instant?

    public init(
        service: any GemStreamServiceProtocol,
        webSocket: any WebSocketConnectable,
        health: ConnectionComponentHealth,
        reconnection: any Reconnectable,
    ) {
        self.service = service
        self.webSocket = webSocket
        self.health = health
        self.reconnection = reconnection
    }

    deinit {
        observeTask?.cancel()
    }

    // MARK: - Public API

    public func connect() {
        isActive = true
        startObserving()
    }

    public func updateSession() async {
        startObserving()
        do {
            try await service.updateSession()
        } catch {
            debugLog("stream session update error: \(error)")
        }
    }

    public func disconnect() {
        isActive = false
        observeTask?.cancel()
    }

    // MARK: - Private

    private func startObserving() {
        guard isActive, observeTask == nil else { return }
        observeTask = Task { [weak self] in
            await self?.observeConnection()
        }
    }

    private func observeConnection() async {
        let failed: Bool
        do {
            try await runConnection()
            failed = false
        } catch is CancellationError {
            failed = false
        } catch {
            debugLog("stream connection error: \(error)")
            failed = true
        }
        health.report(isHealthy: false)
        await webSocket.disconnect()
        await service.disconnected()
        if failed {
            let next = reconnection.reconnection(attempt: failedAttempts, connectedFor: connectedAt.map { $0.duration(to: .now) } ?? .zero)
            failedAttempts = next.nextAttempt
            try? await Task.sleep(for: next.delay)
        }
        connectedAt = nil
        observeTask = nil
        if Task.isCancelled || failed {
            startObserving()
        }
    }

    private func runConnection() async throws {
        let shouldConnect = try await service.prepareConnection()
        try Task.checkCancellation()
        debugLog("stream connecting: \(shouldConnect)")
        guard shouldConnect else { return }
        for await event in await webSocket.connect() {
            try Task.checkCancellation()
            switch event {
            case .connected:
                debugLog("stream connected")
                try await service.connected()
                connectedAt = .now
                health.report(isHealthy: true)
            case let .message(data):
                await onMessage(data)
            case .disconnected:
                debugLog("stream disconnected")
                if isActive {
                    health.report(isHealthy: false)
                }
                await service.disconnected()
            }
        }
    }

    private func onMessage(_ data: Data) async {
        do {
            let event = try await service.decodeEvent(event: String(decoding: data, as: UTF8.self))
            debugLog("stream event: \(event)")
            Task { [service] in
                do {
                    try await service.sync(event: event)
                } catch {
                    debugLog("stream sync error: \(error)")
                }
            }
        } catch {
            debugLog("stream dropped an event: \(error)")
        }
    }
}
