// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone

public actor NativeProvider {
    let session: URLSession

    init(session: URLSession = .shared) {
        self.session = session
    }
}

public final class MemoryPreferences: GemPreferencesStore, @unchecked Sendable {
    private var values: [String: String] = [:]
    private let lock = NSLock()

    public func get(key: String) -> String? {
        lock.withLock { values[key] }
    }

    public func set(key: String, value: String) throws {
        lock.withLock { values[key] = value }
    }

    public func remove(key: String) throws {
        lock.withLock { _ = values.removeValue(forKey: key) }
    }

    public func clear() throws {
        lock.withLock { values.removeAll() }
    }
}

extension NativeProvider: AlienProvider {
    public func request(target: Gemstone.AlienTarget) async throws -> Gemstone.AlienResponse {
        print("==> handle request: \(target)")

        let (data, response) = try await self.session.data(for: target.asRequest())
        let status = (response as? HTTPURLResponse)?.statusCode

        print("<== response size: \(data.count)")

        return Gemstone.AlienResponse(status: status.map(UInt16.init), data: data)
    }
}
