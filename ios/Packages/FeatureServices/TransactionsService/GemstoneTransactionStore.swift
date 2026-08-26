// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.AssetId
import typealias Gemstone.Transaction
import protocol Gemstone.GemTransactionStore
import GemstonePrimitives
import Preferences
import Primitives
import Store

public final class GemstoneTransactionStore: GemTransactionStore, @unchecked Sendable {
    private let store: TransactionStore

    public init(store: TransactionStore) {
        self.store = store
    }

    public func getSyncTimestamp(walletId: String, assetId: Gemstone.AssetId?) async throws -> UInt64 {
        let preferences = try WalletPreferences(walletId: WalletId.from(id: walletId))
        let timestamp = assetId.map { preferences.transactionsForAssetTimestamp(assetId: $0) } ?? preferences.transactionsTimestamp
        return UInt64(timestamp)
    }

    public func setSyncTimestamp(walletId: String, assetId: Gemstone.AssetId?, timestamp: UInt64) async throws {
        let preferences = try WalletPreferences(walletId: WalletId.from(id: walletId))
        if let assetId {
            preferences.setTransactionsForAssetTimestamp(assetId: assetId, value: Int(timestamp))
        } else {
            preferences.transactionsTimestamp = Int(timestamp)
        }
    }

    public func addTransactions(walletId: String, transactions: [Gemstone.Transaction]) async throws {
        try store.addTransactions(walletId: WalletId.from(id: walletId), transactions: transactions.map { try Primitives.Transaction($0) })
    }
}
