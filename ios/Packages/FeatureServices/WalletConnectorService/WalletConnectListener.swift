// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemWalletConnectEvent
import protocol Gemstone.GemWalletConnectListener

public final class WalletConnectListener: GemWalletConnectListener, Sendable {
    public let events: AsyncStream<GemWalletConnectEvent>
    private let continuation: AsyncStream<GemWalletConnectEvent>.Continuation

    public init() {
        (events, continuation) = AsyncStream.makeStream()
    }

    public func onEvent(event: GemWalletConnectEvent) {
        continuation.yield(event)
    }
}
