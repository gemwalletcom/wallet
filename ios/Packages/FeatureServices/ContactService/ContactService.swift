// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemContactServiceProtocol
import GemstonePrimitives
import LocalStore
import Primitives

public struct ContactService: Sendable {
    private let provider: any GemContactServiceProtocol
    private let localStore = LocalStore()

    public init(provider: any GemContactServiceProtocol) {
        self.provider = provider
    }

    public func addContact(_ contact: Contact, addresses: [ContactAddress]) async throws {
        try await provider.addContact(contact: contact.json(), addresses: addresses.map { try $0.json() })
    }

    public func updateContact(_ contact: Contact, addresses: [ContactAddress]) async throws {
        try await provider.updateContact(contact: contact.json(), addresses: addresses.map { try $0.json() })
    }

    public func deleteContact(_ contact: Contact) async throws {
        try await provider.deleteContact(contactId: contact.id)
        if let imageUrl = contact.imageUrl {
            try? localStore.remove(imageUrl)
        }
    }

    public func saveAvatar(_ data: Data) throws -> String {
        try localStore.store(data, id: UUID().uuidString, documentType: "png")
    }

    public func removeAvatar(_ fileName: String) throws {
        try localStore.remove(fileName)
    }
}
