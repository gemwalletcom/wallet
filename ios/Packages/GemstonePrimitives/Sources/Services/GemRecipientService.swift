// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemRecipientSection
import protocol Gemstone.GemRecipientServiceProtocol
import Primitives

public extension GemRecipientServiceProtocol {
    func recipientSections(wallets: [Wallet], chain: Chain, hasContacts: Bool) -> [GemRecipientSection] {
        recipientSections(wallets: wallets.map { $0.toGem() }, chain: chain.rawValue, hasContacts: hasContacts)
    }
}
