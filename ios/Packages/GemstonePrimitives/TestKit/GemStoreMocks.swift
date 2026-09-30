// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import enum Gemstone.GemNameInputStep
import struct Gemstone.GemPriceAlertSession
import GemstonePrimitives
import Primitives

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

final class GemNodeStoreMock: GemNodeStore, @unchecked Sendable {
    init() {}

    func getNodes(chain _: Gemstone.Chain) async throws -> [Gemstone.Node] {
        []
    }

    func addNode(chain _: Gemstone.Chain, node _: Gemstone.Node) async throws {}

    func deleteNode(chain _: Gemstone.Chain, url _: String) async throws {}
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

final class StubAlienProvider: AlienProvider, @unchecked Sendable {
    init() {}

    func request(target _: AlienTarget) async throws -> AlienResponse {
        throw AnyError("StubAlienProvider does not perform requests")
    }
}

final class GemContactStoreMock: GemContactStore, @unchecked Sendable {
    init() {}

    func getAddresses(contactId _: String) async throws -> [Gemstone.ContactAddress] {
        []
    }

    func saveContact(contact _: Gemstone.Contact, addresses _: [Gemstone.ContactAddress]) async throws {}

    func updateContact(contact _: Gemstone.Contact, addresses _: [Gemstone.ContactAddress], deleteAddressIds _: [String]) async throws {}

    func deleteContact(contactId _: String) async throws {}
}

final class GemAddressStoreMock: GemAddressStore, @unchecked Sendable {
    init() {}

    func getAddressName(chain _: Gemstone.Chain, address _: String) throws -> Gemstone.AddressName? {
        nil
    }

    func saveAddressNames(updates _: [GemAddressNameUpdate]) async throws {}

    func deleteAddressNames(names _: [Gemstone.AddressName]) async throws {}
}

final class GemFileStoreMock: GemFileStore, @unchecked Sendable {
    init() {}

    func saveFile(data _: Data, extension _: String) throws -> String {
        ""
    }

    func saveNamedFile(data _: Data, fileName: String) throws -> String {
        fileName
    }

    func exists(fileName _: String) -> Bool {
        false
    }

    func path(fileName: String) -> String {
        fileName
    }

    func remove(fileName _: String) throws {}
}

final class GemWalletPreferencesStoreMock: GemWalletPreferencesStore, @unchecked Sendable {
    private let lock = NSLock()
    private var values: [String: String] = [:]

    init() {}

    func get(walletId: Gemstone.WalletId, key: String) -> String? {
        lock.withLock { values["\(walletId):\(key)"] }
    }

    func set(walletId: Gemstone.WalletId, key: String, value: String) throws {
        lock.withLock { values["\(walletId):\(key)"] = value }
    }

    func deletePreferences(walletId: Gemstone.WalletId) throws {
        lock.withLock { values = values.filter { !$0.key.hasPrefix("\(walletId):") } }
    }
}

public extension GemWalletPreferencesService {
    static func mock() -> GemWalletPreferencesService {
        GemWalletPreferencesService(store: GemWalletPreferencesStoreMock())
    }
}
