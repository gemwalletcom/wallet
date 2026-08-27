// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemContactServiceProtocol
import Primitives

public extension GemContactServiceProtocol {
    func addContact(_ contact: Contact, addresses: [ContactAddress]) async throws {
        try await addContact(contact: contact.json(), addresses: addresses.map { try $0.json() })
    }

    func updateContact(_ contact: Contact, addresses: [ContactAddress]) async throws {
        try await updateContact(contact: contact.json(), addresses: addresses.map { try $0.json() })
    }

    func deleteContact(_ contact: Contact) async throws {
        try await deleteContact(contact: contact.json())
    }
}
