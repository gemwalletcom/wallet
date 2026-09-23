// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemNotificationsServiceProtocol
import enum Gemstone.GemPushResult
import struct Gemstone.GemPushState

public final class GemNotificationsServiceMock: GemNotificationsServiceProtocol, @unchecked Sendable {
    public var state: GemPushState
    public var offersForSupport = false
    public private(set) var requested: [Bool] = []

    public init(state: GemPushState = GemPushState(isEnabled: false, result: .stored)) {
        self.state = state
    }

    public func isEnabled() -> Bool {
        state.isEnabled
    }

    public func setEnabled(enabled: Bool) async -> GemPushState {
        requested.append(enabled)
        return state
    }

    public func enableForSupport() async -> GemPushState? {
        guard offersForSupport else { return nil }
        return await setEnabled(enabled: true)
    }
}
