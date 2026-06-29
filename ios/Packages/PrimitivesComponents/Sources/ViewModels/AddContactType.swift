// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

public enum AddContactType: Hashable, Sendable {
    case new(ChainRecipient)
    case existing(ChainRecipient)

    public var id: String {
        switch self {
        case let .new(recipient): "new-\(recipient.chain.rawValue)-\(recipient.recipient.address)"
        case let .existing(recipient): "existing-\(recipient.chain.rawValue)-\(recipient.recipient.address)"
        }
    }
}
