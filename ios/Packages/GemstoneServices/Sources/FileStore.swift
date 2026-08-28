// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemFileStore
import LocalStore

public final class GemstoneFileStore: GemFileStore, Sendable {
    private let store = LocalStore()

    public init() {}

    public func saveFile(data: Data, extension: String) throws -> String {
        try store.store(data, id: UUID().uuidString, documentType: `extension`)
    }

    public func saveNamedFile(data: Data, fileName: String) throws -> String {
        try store.store(data, fileName: fileName).path()
    }

    public func exists(fileName: String) -> Bool {
        store.exists(fileName)
    }

    public func path(fileName: String) -> String {
        store.url(for: fileName).path()
    }

    public func remove(fileName: String) throws {
        try store.remove(fileName)
    }
}
