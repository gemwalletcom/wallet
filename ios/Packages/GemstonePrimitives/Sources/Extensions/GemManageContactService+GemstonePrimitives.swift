// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemManageContactServiceProtocol
import struct Gemstone.GemContactAddressInput
import enum Gemstone.GemContactAvatar
import struct Gemstone.GemContactInput
import Primitives

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

    func updateContact(_ contact: Contact, addresses: [ContactAddress]) async throws {
        try await updateContact(contact: contact.json(), addresses: addresses.map { $0.json() })
    }

    func addAddress(
        _ addresses: [ContactAddress],
        contactId: String,
        chain: Chain,
        address: String,
        memo: String?,
        replacingId: String?,
    ) throws -> [ContactAddress] {
        try addAddress(
            addresses: addresses.map { $0.json() },
            input: GemContactAddressInput(
                contactId: contactId,
                chain: chain.rawValue,
                address: address,
                memo: memo,
                replacingId: replacingId,
            ),
        ).map { try ContactAddress($0) }
    }

    func deleteContact(_ contact: Contact) async throws {
        try await deleteContact(contact: contact.json())
    }

    var defaultContactChain: Chain {
        guard let chain = Chain(rawValue: defaultChain()) else {
            preconditionFailure("Undecodable default contact chain: \(defaultChain())")
        }
        return chain
    }
}
