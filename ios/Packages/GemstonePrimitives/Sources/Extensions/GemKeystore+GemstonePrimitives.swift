// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import Primitives

public extension GemKeystoreAccount {
    func mapToAccount() -> Primitives.Account {
        Primitives.Account(
            chain: Primitives.Chain(core: chain),
            address: address,
            derivationPath: derivationPath,
            extendedPublicKey: publicKey ?? "",
        )
    }
}

public extension GemStoredWallet {
    func mapToWallet(name: String, source: Primitives.WalletSource) throws -> Primitives.Wallet {
        // externalId stays nil for v4 wallets
        try Primitives.Wallet(
            id: Primitives.WalletId.from(id: walletId),
            externalId: nil,
            name: name,
            index: 0,
            type: walletType.map(),
            accounts: accounts.map { $0.mapToAccount() },
            isPinned: false,
            imageUrl: nil,
            source: source,
        )
    }
}
