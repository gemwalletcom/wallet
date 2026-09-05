// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemPaymentRecipient
import struct Gemstone.GemRecipient
import GemstonePrimitives

public extension GemPaymentRecipient {
    static func mock(
        recipient: GemRecipient = .mock(),
        amount: String? = nil,
    ) -> GemPaymentRecipient {
        GemPaymentRecipient(recipient: recipient, amount: amount)
    }
}
