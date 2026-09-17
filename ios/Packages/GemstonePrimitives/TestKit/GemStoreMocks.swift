// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import struct Gemstone.GemPriceAlertSession
import enum Gemstone.GemNameInputStep

public final class GemSecureStoreMock: GemSecureStore, @unchecked Sendable {
    private var values: [String: String] = [:]

    public init() {}

    public func get(key: String) throws -> String? {
        values[key]
    }

    public func set(key: String, value: String) throws {
        values[key] = value
    }

    public func remove(key: String) throws {
        values[key] = nil
    }
}

public final class GemNodeStoreMock: GemNodeStore, @unchecked Sendable {
    public init() {}

    public func getNodes(chain _: Gemstone.Chain) async throws -> [Gemstone.Node] { [] }

    public func addNode(chain _: Gemstone.Chain, node _: Gemstone.Node) async throws {}

    public func deleteNode(chain _: Gemstone.Chain, url _: String) async throws {}
}

public extension GemNodeService {
    static func mock() -> GemNodeService {
        GemNodeService(store: GemNodeStoreMock(), preferences: GemPreferencesStoreMock())
    }
}

public final class GemPreferencesStoreMock: GemPreferencesStore, @unchecked Sendable {
    private let lock = NSLock()
    private var values: [String: String] = [:]

    public init() {}

    public func get(key: String) -> String? {
        lock.withLock { values[key] }
    }

    public func set(key: String, value: String) throws {
        lock.withLock { values[key] = value }
    }

    public func remove(key: String) throws {
        lock.withLock { values[key] = nil }
    }

    public func clear() throws {
        lock.withLock { values.removeAll() }
    }
}

public final class StubAlienProvider: AlienProvider, @unchecked Sendable {
    public init() {}

    public func request(target _: AlienTarget) async throws -> AlienResponse {
        throw AnyError("StubAlienProvider does not perform requests")
    }

}

public final class GemContactStoreMock: GemContactStore, @unchecked Sendable {
    public init() {}

    public func getAddresses(contactId _: String) async throws -> [Gemstone.ContactAddress] {
        []
    }

    public func saveContact(contact _: Gemstone.Contact, addresses _: [Gemstone.ContactAddress]) async throws {}

    public func updateContact(contact _: Gemstone.Contact, addresses _: [Gemstone.ContactAddress], deleteAddressIds _: [String]) async throws {}

    public func deleteContact(contactId _: String) async throws {}
}

public final class GemAddressStoreMock: GemAddressStore, @unchecked Sendable {
    public init() {}

    public func getAddressName(chain _: Gemstone.Chain, address _: String) throws -> Gemstone.AddressName? {
        nil
    }

    public func saveAddressNames(names _: [Gemstone.AddressName]) async throws {}

    public func deleteAddressNames(names _: [Gemstone.AddressName]) async throws {}
}

public final class GemFileStoreMock: GemFileStore, @unchecked Sendable {
    public init() {}

    public func saveFile(data _: Data, extension _: String) throws -> String {
        ""
    }

    public func saveNamedFile(data _: Data, fileName: String) throws -> String {
        fileName
    }

    public func exists(fileName _: String) -> Bool {
        false
    }

    public func path(fileName: String) -> String {
        fileName
    }

    public func remove(fileName _: String) throws {}
}

public final class GemWalletPreferencesStoreMock: GemWalletPreferencesStore, @unchecked Sendable {
    private let lock = NSLock()
    private var values: [String: String] = [:]

    public init() {}

    public func get(walletId: Gemstone.WalletId, key: String) -> String? {
        lock.withLock { values["\(walletId):\(key)"] }
    }

    public func set(walletId: Gemstone.WalletId, key: String, value: String) throws {
        lock.withLock { values["\(walletId):\(key)"] = value }
    }

    public func deletePreferences(walletId: Gemstone.WalletId) throws {
        lock.withLock { values = values.filter { !$0.key.hasPrefix("\(walletId):") } }
    }
}

public extension GemWalletPreferencesService {
    static func mock() -> GemWalletPreferencesService {
        GemWalletPreferencesService(store: GemWalletPreferencesStoreMock())
    }
}
