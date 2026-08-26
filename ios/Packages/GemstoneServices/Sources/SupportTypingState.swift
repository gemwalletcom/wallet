// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

@Observable
public final class SupportTypingState: Sendable {
    @MainActor public private(set) var agent: SupportAgent?

    public init() {}

    @MainActor
    public func update(_ typing: SupportTyping) {
        switch typing.status {
        case .on:
            agent = typing.agent
        case .off:
            clear()
        }
    }

    @MainActor
    public func clear() {
        agent = nil
    }
}
