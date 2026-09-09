// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

final class FormatterCache<Key: Hashable, Value>: @unchecked Sendable {
    private let lock = NSLock()
    private var values: [Key: Value] = [:]

    func value(for key: Key, make: () -> Value) -> Value {
        lock.withLock {
            if let value = values[key] {
                return value
            }
            let value = make()
            values[key] = value
            return value
        }
    }
}
