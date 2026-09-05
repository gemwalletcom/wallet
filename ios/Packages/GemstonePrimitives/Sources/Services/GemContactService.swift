// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemContactAddressInput
import enum Gemstone.GemContactAvatar
import struct Gemstone.GemContactInput
import protocol Gemstone.GemContactServiceProtocol
import protocol Gemstone.GemManageContactServiceProtocol
import Primitives

public extension GemContactServiceProtocol {
    func updateContact(_ contact: Contact, addresses: [ContactAddress]) async throws {
        try await updateContact(contact: contact.map(), addresses: addresses.map { $0.map() })
    }

    func deleteContact(_ contact: Contact) async throws {
        try await deleteContact(contact: contact.map())
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
        try await saveContact(
            input: GemContactInput(
                id: id,
                existing: existing?.map(),
                name: name,
                description: description,
                avatar: avatar,
                addresses: addresses.map { $0.map() },
            )
        ).map()
    }

    var defaultContactChain: Chain {
        Chain(core: defaultChain())
    }
}

public extension GemContactAddressInput {
    init(contactId: String, chain: Chain, address: String, memo: String?, replacingId: String?) {
        self.init(contactId: contactId, chain: chain.rawValue, address: address, memo: memo, replacingId: replacingId)
    }

    func addAddress(_ addresses: [ContactAddress]) -> [ContactAddress] {
        addAddress(addresses: addresses.map { $0.map() }).map { $0.map() }
    }
}
