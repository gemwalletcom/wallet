// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

public extension WalletConnectSessionProposal {
    static func mock(
        defaultWallet: Wallet = .mock(),
        wallets: [Wallet] = [.mock()],
        metadata: WalletConnectAppMetadata = .mock(),
    ) -> WalletConnectSessionProposal {
        WalletConnectSessionProposal(
            defaultWallet: defaultWallet,
            wallets: wallets,
            metadata: metadata,
        )
    }
}
