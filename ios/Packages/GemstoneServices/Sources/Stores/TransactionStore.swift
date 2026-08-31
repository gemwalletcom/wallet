// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.AssetId
import typealias Gemstone.Transaction
import protocol Gemstone.GemTransactionStore
import GemstonePrimitives
import Primitives
import Store

public final class GemstoneTransactionStore: GemTransactionStore, @unchecked Sendable {
    private let store: TransactionStore

    public init(store: TransactionStore) {
        self.store = store
    }

    public func saveTransactions(walletId: String, transactions: [Gemstone.Transaction]) async throws {
        try store.addTransactions(walletId: WalletId.from(id: walletId), transactions: transactions.map { try Primitives.Transaction($0) })
    }
}
