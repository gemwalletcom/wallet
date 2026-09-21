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
    private var observeTask: Task<Void, Never>?
    private var isActive = false

    public init(
        service: any GemStreamServiceProtocol,
        webSocket: any WebSocketConnectable,
        health: ConnectionComponentHealth,
    ) {
        self.service = service
        self.webSocket = webSocket
        self.health = health
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
        do {
            let shouldConnect = try await service.prepareConnection()
            try Task.checkCancellation()
            debugLog("stream connecting: \(shouldConnect)")
            if shouldConnect {
                for await event in await webSocket.connect() {
                    try Task.checkCancellation()
                    await onSocketEvent(event)
                }
            }
        } catch is CancellationError {
        } catch {
            debugLog("stream connection error: \(error)")
        }
        await webSocket.disconnect()
        await service.disconnected()
        observeTask = nil
        if Task.isCancelled {
            startObserving()
        }
    }

    private func onSocketEvent(_ event: WebSocketEvent) async {
        do {
            switch event {
            case .connected:
                debugLog("stream connected")
                health.report(isHealthy: true)
                try await service.connected()
            case let .message(data):
                let event = try await service.decodeEvent(event: String(decoding: data, as: UTF8.self))
                debugLog("stream event: \(event)")
                Task { [service] in
                    do {
                        try await service.sync(event: event)
                    } catch {
                        debugLog("stream sync error: \(error)")
                    }
                }
            case .disconnected:
                debugLog("stream disconnected")
                if isActive {
                    health.report(isHealthy: false)
                }
                await service.disconnected()
            }
        } catch {
            debugLog("stream dropped an event: \(error)")
        }
    }
}
