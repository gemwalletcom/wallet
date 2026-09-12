// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemStreamServiceProtocol
import GemstonePrimitives
import Primitives
import WebSocketClient

public actor StreamObserverService: Sendable {
    private let service: any GemStreamServiceProtocol
    private let webSocket: any WebSocketConnectable
    private var observeTask: Task<Void, Never>?
    private var isActive = false

    public init(
        service: any GemStreamServiceProtocol,
        webSocket: any WebSocketConnectable,
    ) {
        self.service = service
        self.webSocket = webSocket
    }

    deinit {
        observeTask?.cancel()
    }

    // MARK: - Public API

    public func connect() {
        guard !isActive else { return }
        isActive = true
        restart()
    }

    public func updateSession() async {
        if await webSocket.state == .disconnected {
            restart()
        }
        do {
            try await service.updateSession()
        } catch {
            debugLog("stream session update error: \(error)")
        }
    }

    public func disconnect() async {
        isActive = false
        observeTask?.cancel()
        await observeTask?.value
        observeTask = nil
    }

    // MARK: - Private

    private func restart() {
        guard isActive else { return }
        let previous = observeTask
        previous?.cancel()
        observeTask = Task { [weak self] in
            await previous?.value
            guard !Task.isCancelled else { return }
            await self?.observeConnection()
        }
    }

    private func observeConnection() async {
        do {
            let shouldConnect = try await service.prepareConnection()
            try Task.checkCancellation()
            if shouldConnect {
                for await event in await webSocket.connect() {
                    try Task.checkCancellation()
                    await handle(event)
                }
            }
        } catch is CancellationError {
        } catch {
            debugLog("stream connection error: \(error)")
        }
        await webSocket.disconnect()
        await service.disconnected()
    }

    private func handle(_ event: WebSocketEvent) async {
        do {
            switch event {
            case .connected: try await service.connected()
            case let .message(data):
                let event = try await service.handle(event: String(decoding: data, as: UTF8.self))
                debugLog("stream event: \(event)")
            case .disconnected: await service.disconnected()
            }
        } catch {
            debugLog("stream event handler error: \(error)")
        }
    }
}
