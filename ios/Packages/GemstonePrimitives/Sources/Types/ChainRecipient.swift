// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemRecipient
import Primitives

public struct ChainRecipient: Sendable, Hashable {
    public let recipient: GemRecipient
    public let chain: Chain

    public init(recipient: GemRecipient, chain: Chain) {
        self.recipient = recipient
        self.chain = chain
    }
}
