// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public extension Contact {
    static func new(
        id: String,
        name: String,
        description: String?,
        createdAt: Date = .now,
    ) -> Contact {
        Contact(
            id: id,
            name: name,
            description: description,
            createdAt: createdAt,
            updatedAt: .now,
        )
    }
}

public extension ContactData {
    func addAddress(from recipient: ChainRecipient) -> ContactData {
        let address = ContactAddress.new(
            contactId: contact.id,
            chain: recipient.chain,
            address: recipient.recipient.address,
            memo: recipient.recipient.memo,
        )
        guard !addresses.contains(where: { $0.id == address.id }) else {
            return self
        }
        return ContactData(contact: contact, addresses: addresses + [address])
    }
}

public extension ContactAddress {
    static func new(
        contactId: String,
        chain: Chain,
        address: String,
        memo: String?,
    ) -> ContactAddress {
        ContactAddress(
            id: contactId + "_" + chain.rawValue + "_" + address,
            contactId: contactId,
            address: address,
            chain: chain,
            memo: memo,
        )
    }
}
