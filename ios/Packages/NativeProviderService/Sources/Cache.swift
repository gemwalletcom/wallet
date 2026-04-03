// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

actor MemoryCache {
    private struct Entry {
        let data: Data
        let expiresAt: ContinuousClock.Instant
    }

    private var map = [String: Entry]()

    func set(key: String, value: Data, ttl: Duration) {
        map[key] = Entry(data: value, expiresAt: .now + ttl)
    }

    func get(key: String) -> Data? {
        guard let entry = map[key] else { return nil }
        if ContinuousClock.now >= entry.expiresAt {
            map[key] = nil
            return nil
        }
        return entry.data
    }
}
