// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemPerpetualSubscription
import class Gemstone.HyperliquidSubscriptions
import Primitives
import WebSocketClient

public actor HyperliquidSubscriptionService {
    private let webSocket: any WebSocketConnectable
    private let subscriptions = HyperliquidSubscriptions()

    public init(webSocket: any WebSocketConnectable) {
        self.webSocket = webSocket
    }

    public func subscribe(_ subscription: GemPerpetualSubscription) async throws {
        try await send(subscriptions.subscribe(subscription: subscription))
    }

    public func unsubscribe(_ subscription: GemPerpetualSubscription) async throws {
        try await send(subscriptions.unsubscribe(subscription: subscription))
    }

    public func connected(address: String, mode: PerpetualAccountMode) async throws {
        try await send(subscriptions.connected(address: address, mode: mode.map()))
    }

    public func disconnected() {
        subscriptions.disconnected()
    }

    // MARK: - Private

    private func send(_ requests: [String]) async throws {
        for request in requests {
            try await webSocket.send(request)
        }
    }
}
