// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

@propertyWrapper
public final class Locked<Value>: @unchecked Sendable {
    private var value: Value
    private let lock = NSLock()

    public init(wrappedValue: Value) {
        value = wrappedValue
    }

    public var wrappedValue: Value {
        get { lock.withLock { value } }
        set { lock.withLock { value = newValue } }
    }

    public func withLock<T>(_ body: (inout Value) throws -> T) rethrows -> T {
        try lock.withLock { try body(&value) }
    }
}

public struct UncheckedSendable<Value>: @unchecked Sendable {
    public let value: Value

    public init(value: Value) {
        self.value = value
    }
}
