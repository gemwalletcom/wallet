// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemAddressService
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
        let entries = contacts.flatMap { contactData in
            contactData.addresses.map { (contactData.contact.name, $0) }
        }
        let subtitles = GemAddressService.shared.formatAll(
            addresses: entries.map { ChainAddress(chain: $0.1.chain, address: $0.1.address).toGem() },
            style: .short,
        )
        return zip(entries, subtitles).map { entry, subtitle in
            ListItemValue(
                title: entry.0,
                subtitle: subtitle,
                value: GemRecipient(address: entry.1.address, name: entry.0, memo: entry.1.memo),
            )
        }
    }
}
