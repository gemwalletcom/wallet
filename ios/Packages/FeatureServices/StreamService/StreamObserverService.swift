// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemStreamServiceProtocol
import protocol Gemstone.GemStreamSubscriptionServiceProtocol
import GemstonePrimitives
import Preferences
import Primitives
import WebSocketClient

public actor StreamObserverService: Sendable {
    private let subscriptionService: any GemStreamSubscriptionServiceProtocol
    private let service: any GemStreamServiceProtocol
    private let preferences: Preferences
    private let webSocket: any WebSocketConnectable
    private var observeTask: Task<Void, Never>?

    public init(
        subscriptionService: any GemStreamSubscriptionServiceProtocol,
        service: any GemStreamServiceProtocol,
        preferences: Preferences,
        webSocket: any WebSocketConnectable,
    ) {
        self.subscriptionService = subscriptionService
        self.service = service
        self.preferences = preferences
        self.webSocket = webSocket
    }

    deinit {
        observeTask?.cancel()
    }

    // MARK: - Public API

    public func connect() {
        guard observeTask == nil else { return }
        observeTask = Task { [weak self] in
            guard let self else { return }
            await observeConnection()
        }
    }

    public func disconnect() async {
        guard observeTask != nil else { return }
        observeTask?.cancel()
        observeTask = nil
        subscriptionService.reset()
        await webSocket.disconnect()
    }

    // MARK: - Private

    private func observeConnection() async {
        for await event in await webSocket.connect() {
            guard !Task.isCancelled else { break }
            switch event {
            case .connected: await resubscribe()
            case let .message(data): await handleMessage(data)
            case .disconnected: subscriptionService.reset()
            }
        }
    }

    private func resubscribe() async {
        do {
            try await subscriptionService.resubscribe()
        } catch {
            debugLog("stream subscription: resubscribe failed: \(error)")
        }
    }

    private func handleMessage(_ data: Data) async {
        do {
            try await service.handle(event: String(decoding: data, as: UTF8.self), currency: Currency(id: preferences.currency).json())
        } catch {
            debugLog("stream event handler error: \(error)")
        }
    }
}
