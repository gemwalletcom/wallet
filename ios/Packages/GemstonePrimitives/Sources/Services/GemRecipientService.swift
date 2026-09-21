// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemRecipient
import struct Gemstone.GemRecipientSection
import protocol Gemstone.GemRecipientServiceProtocol
import Primitives

public extension GemRecipientServiceProtocol {
    func recipientSections(wallets: [Wallet], chain: Chain, contacts: [GemRecipient]) -> [GemRecipientSection] {
        recipientSections(wallets: wallets.map { $0.toGem() }, chain: chain.rawValue, contacts: contacts)
    }
}
