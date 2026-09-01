// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import os
import Primitives

@Observable
public final class ObservableSupportTyping: Sendable {
    @ObservationIgnored
    private let state = OSAllocatedUnfairLock<SupportAgent?>(initialState: .none)

    public var agent: SupportAgent? {
        access(keyPath: \.agent)
        return state.withLock { $0 }
    }

    public init() {}

    public func update(_ typing: SupportTyping) {
        switch typing.status {
        case .on: set(typing.agent)
        case .off: clear()
        }
    }

    public func clear() {
        set(.none)
    }

    private func set(_ agent: SupportAgent?) {
        withMutation(keyPath: \.agent) {
            state.withLock { $0 = agent }
        }
    }
}
