// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemContactAddressInput
import enum Gemstone.GemContactAvatar
import struct Gemstone.GemContactInput
import protocol Gemstone.GemContactsServiceProtocol
import protocol Gemstone.GemManageContactServiceProtocol
import Primitives

public extension GemContactsServiceProtocol {
    func addAddress(
        _ addresses: [ContactAddress],
        contactId: String,
        chain: Chain,
        address: String,
        memo: String?,
        replacingId: String?,
    ) throws -> [ContactAddress] {
        try addAddress(addresses: addresses.map { $0.json() }, input: .input(contactId: contactId, chain: chain, address: address, memo: memo, replacingId: replacingId))
            .map { try ContactAddress($0) }
    }

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

    func addAddress(
        _ addresses: [ContactAddress],
        contactId: String,
        chain: Chain,
        address: String,
        memo: String?,
        replacingId: String?,
    ) throws -> [ContactAddress] {
        try addAddress(addresses: addresses.map { $0.json() }, input: .input(contactId: contactId, chain: chain, address: address, memo: memo, replacingId: replacingId))
            .map { try ContactAddress($0) }
    }

    var defaultContactChain: Chain {
        Chain.decoded(defaultChain())
    }
}


private extension GemContactAddressInput {
    static func input(contactId: String, chain: Chain, address: String, memo: String?, replacingId: String?) -> GemContactAddressInput {
        GemContactAddressInput(contactId: contactId, chain: chain.rawValue, address: address, memo: memo, replacingId: replacingId)
    }
}

private extension Chain {
    static func decoded(_ rawValue: String) -> Chain {
        guard let chain = Chain(rawValue: rawValue) else {
            preconditionFailure("Undecodable chain: \(rawValue)")
        }
        return chain
    }
}
