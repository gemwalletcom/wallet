// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemRecipient
import func Gemstone.recipientId

extension GemRecipient: @retroactive Identifiable {
    public var id: String {
        recipientId(recipient: self)
    }
}
