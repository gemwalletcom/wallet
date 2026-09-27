// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit
import Store

public extension DB {
    static func mock(
        chains: [Chain] = [],
        wallets: [Wallet] = [],
        assets: [AssetBasic] = [],
        balances: [UpdateBalance] = [],
    ) -> DB {
        let db = DB(fileName: "\(UUID().uuidString).sqlite")
        let natives = Set(chains + wallets.flatMap { $0.accounts.map(\.chain) } + assets.map(\.asset.chain))
            .subtracting(assets.filter { $0.asset.type == .native }.map(\.asset.chain))
        do {
            try AssetStore(db: db).add(assets: natives.map { .mock(asset: .mock(id: $0.assetId)) } + assets)
            for wallet in wallets {
                try WalletStore(db: db).addWallet(wallet)
                try BalanceStore(db: db).addBalance(assets.map { AddBalance(assetId: $0.asset.id, isEnabled: true) }, for: wallet.id)
                try BalanceStore(db: db).updateBalances(balances, for: wallet.id)
            }
        } catch {
            preconditionFailure("DB.mock could not seed the database: \(error)")
        }
        return db
    }
}
