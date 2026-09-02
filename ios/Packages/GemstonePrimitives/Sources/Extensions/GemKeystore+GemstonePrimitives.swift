// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import Primitives

public extension GemKeystoreAccount {
    func mapToAccount() throws -> Primitives.Account {
        try Primitives.Account(
            chain: chain.map(),
            address: address,
            derivationPath: derivationPath,
            extendedPublicKey: publicKey ?? "",
        )
    }
}

public extension GemWalletType {
    func map() -> WalletType {
        switch self {
        case .multicoin: .multicoin
        case .single: .single
        case .privateKey: .privateKey
        case .view: .view
        }
    }
}

public extension WalletType {
    func map() -> GemWalletType {
        switch self {
        case .multicoin: .multicoin
        case .single: .single
        case .privateKey: .privateKey
        case .view: .view
        }
    }
}

public extension GemStoredWallet {
    func mapToWallet(name: String, source: WalletSource) throws -> Primitives.Wallet {
        // externalId stays nil for v4 wallets
        try Primitives.Wallet(
            id: Primitives.WalletId.from(id: walletId),
            externalId: nil,
            name: name,
            index: 0,
            type: walletType.map(),
            accounts: accounts.map { try $0.mapToAccount() },
            isPinned: false,
            imageUrl: nil,
            source: source,
        )
    }
}
