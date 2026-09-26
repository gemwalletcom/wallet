// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemRecipientSection
import protocol Gemstone.GemRecipientServiceProtocol
import Primitives

public extension GemRecipientServiceProtocol {
    func recipientSections(wallets: [Wallet], chain: Chain, contacts: [ContactData]) -> [GemRecipientSection] {
        recipientSections(wallets: wallets.map { $0.toGem() }, chain: chain.rawValue, contacts: contacts.map { $0.toGem() })
    }
}
