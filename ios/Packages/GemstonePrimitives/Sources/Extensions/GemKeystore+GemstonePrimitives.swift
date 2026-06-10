// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import Primitives

public extension GemKeystoreAccount {
    func mapToAccount() throws -> Account {
        try Account(
            chain: chain.map(),
            address: address,
            derivationPath: derivationPath,
            extendedPublicKey: publicKey ?? "",
        )
    }
}

public extension GemWalletType {
    var walletType: WalletType {
        switch self {
        case .multicoin: .multicoin
        case .single: .single
        case .privateKey: .privateKey
        case .view: .view
        }
    }
}

public extension GemWalletImport {
    func mapToPreview() throws -> WalletImport {
        try WalletImport(
            walletId: WalletId.from(id: walletId),
            walletType: walletType.walletType,
            accounts: accounts.map { try $0.mapToAccount() },
        )
    }
}

public extension GemStoredWallet {
    func mapToWallet(name: String, source: WalletSource) throws -> Wallet {
        // externalId stays nil for v4 wallets
        try Wallet(
            id: WalletId.from(id: walletId),
            externalId: nil,
            name: name,
            index: 0,
            type: walletType.walletType,
            accounts: accounts.map { try $0.mapToAccount() },
            order: 0,
            isPinned: false,
            imageUrl: nil,
            source: source,
        )
    }
}

public extension Wallet {
    func adding(accounts newAccounts: [Account]) -> Wallet {
        Wallet(
            id: id,
            externalId: externalId,
            name: name,
            index: index,
            type: type,
            accounts: accounts + newAccounts,
            order: order,
            isPinned: isPinned,
            imageUrl: imageUrl,
            source: source,
        )
    }
}
