// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct ChainRecipient: Sendable, Hashable {
    public let recipient: Recipient
    public let chain: Chain

    public init(recipient: Recipient, chain: Chain) {
        self.recipient = recipient
        self.chain = chain
    }
}
