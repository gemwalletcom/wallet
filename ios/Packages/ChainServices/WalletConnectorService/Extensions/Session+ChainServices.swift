// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import ReownWalletKit

extension Session {
    var asSession: Primitives.WalletConnectSession {
        let sessionChains = namespaces.values
            .flatMap(\.accounts)
            .compactMap(\.chain)

        return WalletConnectSession(
            id: topic,
            sessionId: topic,
            state: .active,
            chains: sessionChains,
            createdAt: .now,
            expireAt: expiryDate,
            metadata: peer.metadata,
        )
    }
}

extension Session.Proposal {
    var messageId: String {
        "proposal-\(id)"
    }
}
