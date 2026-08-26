// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.Contact
import typealias Gemstone.ContactAddress
import protocol Gemstone.GemContactStore
import GemstonePrimitives
import Primitives
import Store

public final class GemstoneContactStore: GemContactStore, @unchecked Sendable {
    private let store: ContactStore

    public init(store: ContactStore) {
        self.store = store
    }

    public func getAddressIds(contactId: String) async throws -> [String] {
        try store.getAddressIds(contactId: contactId)
    }

    public func saveContact(contact: Gemstone.Contact, addresses: [Gemstone.ContactAddress]) async throws {
        try store.addContact(Primitives.Contact(contact), addresses: addresses.map { try Primitives.ContactAddress($0) })
    }

    public func updateContact(contact: Gemstone.Contact, addresses: [Gemstone.ContactAddress], deleteAddressIds: [String]) async throws {
        try store.updateContact(
            Primitives.Contact(contact),
            deleteAddressIds: deleteAddressIds,
            addresses: addresses.map { try Primitives.ContactAddress($0) },
        )
    }

    public func deleteContact(contactId: String) async throws {
        try store.deleteContact(id: contactId)
    }
}
