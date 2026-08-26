// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.FiatTransactionData
import protocol Gemstone.GemFiatStore
import GemstonePrimitives
import Primitives
import Store

public final class GemstoneFiatStore: GemFiatStore, @unchecked Sendable {
    private let store: FiatTransactionStore

    public init(store: FiatTransactionStore) {
        self.store = store
    }

    public func saveTransactions(walletId: String, transactions: [Gemstone.FiatTransactionData]) async throws {
        try store.addTransactions(
            walletId: WalletId.from(id: walletId),
            transactions: transactions.map { try Primitives.FiatTransactionData($0) },
        )
    }
}
