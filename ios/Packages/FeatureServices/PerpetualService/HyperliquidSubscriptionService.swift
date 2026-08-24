// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemPerpetualSubscription
import enum Gemstone.GemSubscriptionMethod
import class Gemstone.Hyperliquid
import Primitives
import WebSocketClient

public actor HyperliquidSubscriptionService {
    private let webSocket: any WebSocketConnectable
    private let hyperliquid = Hyperliquid()

    private var activeSubscriptions: Set<GemPerpetualSubscription> = []

    public init(webSocket: any WebSocketConnectable) {
        self.webSocket = webSocket
    }

    public func subscribe(_ subscription: GemPerpetualSubscription) async throws {
        activeSubscriptions.insert(subscription)
        try await send(method: .subscribe, subscription: subscription)
    }

    public func unsubscribe(_ subscription: GemPerpetualSubscription) async throws {
        activeSubscriptions.remove(subscription)
        try await send(method: .unsubscribe, subscription: subscription)
    }

    public func connected(address: String, mode: PerpetualAccountMode) async throws {
        let subscriptions = (hyperliquid.accountSubscriptions(address: address, mode: mode.map()) + activeSubscriptions.asArray()).distinct()
        try await subscribe(subscriptions)
    }

    // MARK: - Private

    private func subscribe(_ subscriptions: [GemPerpetualSubscription]) async throws {
        try await withThrowingTaskGroup(of: Void.self) { group in
            for subscription in subscriptions {
                group.addTask {
                    try await self.send(method: .subscribe, subscription: subscription)
                }
            }
            try await group.waitForAll()
        }
    }

    private func send(method: GemSubscriptionMethod, subscription: GemPerpetualSubscription) async throws {
        try await webSocket.send(
            hyperliquid.websocketRequest(
                method: method,
                subscription: subscription,
            ),
        )
    }
}
