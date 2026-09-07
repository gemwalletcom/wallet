// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemRecipient
import Primitives

public enum AddContactType: Hashable, Sendable {
    case new(GemRecipient, chain: Chain)
    case existing(GemRecipient, chain: Chain)

    public var id: String {
        switch self {
        case let .new(recipient, chain): "new-\(chain.rawValue)-\(recipient.identifier())"
        case let .existing(recipient, chain): "existing-\(chain.rawValue)-\(recipient.identifier())"
        }
    }
}
