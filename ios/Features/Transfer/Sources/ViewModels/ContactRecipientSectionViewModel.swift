// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemRecipient
import Components
import Foundation
import GemstonePrimitives
import Primitives

struct ContactRecipientSectionViewModel {
    private let contacts: [ContactData]

    init(contacts: [ContactData]) {
        self.contacts = contacts
    }

    var listItems: [ListItemValue<GemRecipient>] {
        contacts.flatMap { contactData in
            contactData.addresses.map { address in
                ListItemValue(
                    title: contactData.contact.name,
                    subtitle: AddressFormatter(address: address.address, chain: address.chain).value(),
                    value: GemRecipient(address: address.address, name: contactData.contact.name, memo: address.memo),
                )
            }
        }
    }
}
