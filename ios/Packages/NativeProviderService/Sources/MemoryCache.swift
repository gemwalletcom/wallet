// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

actor MemoryCache {
    private var map = [String: Data]()

    func get(key: String) -> Data? {
        map[key]
    }

    func set(key: String, value: Data) {
        map[key] = value
    }
}
