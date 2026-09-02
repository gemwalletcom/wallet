// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemRecipient
import GemstonePrimitives

public extension RecipientData {
    static func mock(
        recipient: GemRecipient = .mock(),
        amount: String? = nil,
    ) -> RecipientData {
        RecipientData(
            recipient: recipient,
            amount: amount,
        )
    }
}
