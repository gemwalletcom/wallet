// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemContactAddressInput
import protocol Gemstone.GemContactEditorServiceProtocol
import protocol Gemstone.GemContactServiceProtocol
import Primitives

public extension GemContactServiceProtocol {
    func updateContact(_ contact: Contact, addresses: [ContactAddress]) async throws {
        try await updateContact(contact: contact.toGem(), addresses: addresses.map { $0.toGem() })
    }

    func deleteContact(_ contact: Contact) async throws {
        try await deleteContact(contact: contact.toGem())
    }
}

public extension GemContactEditorServiceProtocol {
    var defaultContactChain: Chain {
        Chain(core: defaultChain())
    }
}

public extension GemContactAddressInput {
    init(contactId: String, chain: Chain, address: String, memo: String?, replacingId: String?) {
        self.init(contactId: contactId, chain: chain.rawValue, address: address, memo: memo, replacingId: replacingId)
    }

    func addAddress(_ addresses: [ContactAddress]) -> [ContactAddress] {
        addAddress(addresses: addresses.map { $0.toGem() }).map { $0.toPrimitives() }
    }
}
