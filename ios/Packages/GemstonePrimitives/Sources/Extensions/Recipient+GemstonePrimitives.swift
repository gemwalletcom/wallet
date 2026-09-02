// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemRecipient

extension GemRecipient: @retroactive Identifiable {
    public var id: String {
        [name ?? "", address, memo ?? ""].joined(separator: "_")
    }
}
