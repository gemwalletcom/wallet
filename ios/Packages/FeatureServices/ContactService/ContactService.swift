// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import LocalStore
import Primitives
import Store

public struct ContactService: Sendable {
    private let store: ContactStore
    private let addressStore: AddressStore
    private let localStore = LocalStore()

    public init(store: ContactStore, addressStore: AddressStore) {
        self.store = store
        self.addressStore = addressStore
    }

    public func addContact(_ contact: Contact, addresses: [ContactAddress]) throws {
        try store.addContact(contact, addresses: addresses)
        try syncAddressNames(contact: contact, addresses: addresses)
    }

    public func updateContact(_ contact: Contact, addresses: [ContactAddress]) throws {
        let existingIds = try store.getAddressIds(contactId: contact.id).asSet()
        let newIds = addresses.map(\.id).asSet()
        let changes = SyncDiff.calculate(primary: .local, local: newIds, remote: existingIds)

        try store.updateContact(contact, deleteAddressIds: changes.toDelete.asArray(), addresses: addresses)
        try syncAddressNames(contact: contact, addresses: addresses)
    }

    public func deleteContact(_ contact: Contact) throws {
        try store.deleteContact(id: contact.id)
        if let imageUrl = contact.imageUrl {
            try? localStore.remove(imageUrl)
        }
    }

    // MARK: - Avatar file

    public func saveAvatar(_ data: Data) throws -> String {
        try localStore.store(data, id: UUID().uuidString, documentType: "png")
    }

    public func removeAvatar(_ fileName: String) throws {
        try localStore.remove(fileName)
    }
}

// MARK: - Private

extension ContactService {
    private func syncAddressNames(contact: Contact, addresses: [ContactAddress]) throws {
        let addressNames = addresses.map {
            AddressName(chain: $0.chain, address: $0.address, name: contact.name, type: .contact, status: .verified, imageUrl: contact.imageUrl)
        }
        try addressStore.addAddressNames(addressNames)
    }
}
