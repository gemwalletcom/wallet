// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemFileStore
import LocalStore

public final class GemstoneFileStore: GemFileStore, Sendable {
    private let store = LocalStore()

    public init() {}

    public func save(data: Data, extension: String) throws -> String {
        try store.store(data, id: UUID().uuidString, documentType: `extension`)
    }

    public func remove(fileName: String) throws {
        try store.remove(fileName)
    }
}
