// Copyright (c). Gem Wallet. All rights reserved.

public import LocalAuthentication
import Foundation

public final class UnlockContextStore: @unchecked Sendable {
    private let lock = NSLock()
    private var context: LAContext?

    public init() {}

    public var current: LAContext? {
        lock.withLock { context }
    }

    public func set(_ context: LAContext) {
        lock.withLock { self.context = context }
    }

    public func clear() {
        lock.withLock {
            context?.invalidate()
            context = nil
        }
    }
}
