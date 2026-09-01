// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemContactAddressInput
import enum Gemstone.GemContactAvatar
import struct Gemstone.GemContactInput
import protocol Gemstone.GemContactsServiceProtocol
import protocol Gemstone.GemManageContactServiceProtocol
import Primitives

public extension GemContactsServiceProtocol {
    func updateContact(_ contact: Contact, addresses: [ContactAddress]) async throws {
        try await updateContact(contact: contact.json(), addresses: addresses.map { $0.json() })
    }

    func deleteContact(_ contact: Contact) async throws {
        try await deleteContact(contact: contact.json())
    }
}

public extension GemManageContactServiceProtocol {
    func saveContact(
        id: String,
        existing: Contact?,
        name: String,
        description: String,
        avatar: GemContactAvatar,
        addresses: [ContactAddress],
    ) async throws -> Contact {
        try Contact(
            await saveContact(
                input: GemContactInput(
                    id: id,
                    existing: existing?.json(),
                    name: name,
                    description: description,
                    avatar: avatar,
                    addresses: addresses.map { $0.json() },
                )
            )
        )
    }

    var defaultContactChain: Chain {
        Chain(core: defaultChain())
    }
}

public extension GemContactAddressInput {
    init(contactId: String, chain: Chain, address: String, memo: String?, replacingId: String?) {
        self.init(contactId: contactId, chain: chain.rawValue, address: address, memo: memo, replacingId: replacingId)
    }

    func addAddress(_ addresses: [ContactAddress]) throws -> [ContactAddress] {
        try addAddress(addresses: addresses.map { $0.json() }).map { try ContactAddress($0) }
    }
}
