// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemRecipient

public extension GemRecipient {
    static func mock(
        name: String? = nil,
        address: String = "0x1234567890123456789012345678901234567890",
        memo: String? = nil,
        references: [String] = [],
    ) -> GemRecipient {
        GemRecipient(
            address: address,
            name: name,
            memo: memo,
            references: references,
        )
    }
}
