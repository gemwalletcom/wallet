// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemRecipient
import struct Gemstone.GemRecipientRow

extension GemRecipient: @retroactive Identifiable {
    public var id: String {
        identifier()
    }
}

extension GemRecipientRow: @retroactive Identifiable {
    public var id: String {
        recipient.id
    }
}
