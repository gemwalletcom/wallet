// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

extension Wallet: Identifiable {}

public extension Wallet {
    var canSign: Bool {
        !isViewOnly
    }

    var isViewOnly: Bool {
        type == .view
    }

    var isMultiCoins: Bool {
        type == .multicoin
    }

    func account(for chain: Chain) throws -> Account {
        guard let account = accounts.filter({ $0.chain == chain }).first else {
            throw AnyError("account not found for chain: \(chain.rawValue)")
        }
        return account
    }
}

/// factory
public extension Wallet {
    static func makeView(name: String, chain: Chain, address: String) -> Wallet {
        let id = WalletId.make(walletType: .view, chain: chain, address: address)
        return Wallet(
            id: id,
            externalId: nil,
            name: name,
            index: 0,
            type: .view,
            accounts: [
                Account(
                    chain: chain,
                    address: address,
                    derivationPath: "",
                    extendedPublicKey: "",
                ),
            ],
            isPinned: false,
            imageUrl: nil,
            source: .import,
        )
    }
}
