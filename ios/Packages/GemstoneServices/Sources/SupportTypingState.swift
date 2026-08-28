// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

@Observable
public final class SupportTypingState: @unchecked Sendable {
    @ObservationIgnored
    private let lock = NSLock()
    @ObservationIgnored
    private var typingAgent: SupportAgent?

    public var agent: SupportAgent? {
        access(keyPath: \.agent)
        return lock.withLock { typingAgent }
    }

    public init() {}

    public func update(_ typing: SupportTyping) {
        switch typing.status {
        case .on:
            setAgent(typing.agent)
        case .off:
            clear()
        }
    }

    public func clear() {
        setAgent(.none)
    }

    private func setAgent(_ agent: SupportAgent?) {
        withMutation(keyPath: \.agent) {
            lock.withLock { typingAgent = agent }
        }
    }
}
