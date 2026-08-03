// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension WalletConnectSession {
    static func mock(
        id: String = .empty,
        sessionId: String = .empty,
        state: WalletConnectionState = .active,
        chains: [Chain] = [.ethereum],
        createdAt: Date = .now,
        expireAt: Date = .distantFuture,
        metadata: WalletConnectAppMetadata = .mock(),
    ) -> WalletConnectSession {
        WalletConnectSession(
            id: id,
            sessionId: sessionId,
            state: state,
            chains: chains,
            createdAt: createdAt,
            expireAt: expireAt,
            metadata: metadata,
        )
    }
}

public extension WalletConnectAppMetadata {
    static func mock(
        name: String = "",
        description: String = "",
        url: String = "",
        icon: String = "",
    ) -> WalletConnectAppMetadata {
        WalletConnectAppMetadata(
            name: name,
            description: description,
            url: url,
            icon: icon,
        )
    }
}
