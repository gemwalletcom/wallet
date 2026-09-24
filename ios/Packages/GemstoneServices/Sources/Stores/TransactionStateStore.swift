// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemPendingTransaction
import protocol Gemstone.GemTransactionStateStore
import struct Gemstone.GemTransactionStateUpdate
import typealias Gemstone.Transaction
import typealias Gemstone.TransactionId
import typealias Gemstone.TransactionState
import GemstonePrimitives
import Primitives
import Store

public final class GemstoneTransactionStateStore: GemTransactionStateStore, @unchecked Sendable {
    private let store: TransactionStore

    public init(store: TransactionStore) {
        self.store = store
    }

    public func getPendingTransactions(states: [Gemstone.TransactionState]) async throws -> [GemPendingTransaction] {
        try store.getTransactions(states: states.map { $0.toPrimitives() }).flatMap { walletId, transactions in
            transactions.map { GemPendingTransaction(walletId: walletId.id, transaction: $0.toGem()) }
        }
    }

    public func getTransaction(walletId: String, transactionId: Gemstone.TransactionId) async throws -> GemPendingTransaction? {
        let walletId = try WalletId.from(id: walletId)
        let transactionId = try Primitives.TransactionId(id: transactionId)
        guard try store.getTransactionState(walletId: walletId, transactionId: transactionId) != nil else { return nil }
        let transaction = try store.getTransaction(walletId: walletId, transactionId: transactionId).transaction
        return GemPendingTransaction(walletId: walletId.id, transaction: transaction.toGem())
    }

    public func addTransactions(walletId: String, transactions: [Gemstone.Transaction]) async throws {
        try store.addTransactions(walletId: WalletId.from(id: walletId), transactions: transactions.map(\.transactionAssets))
    }

    public func getState(walletId: String, transactionId: Gemstone.TransactionId) async throws -> Gemstone.TransactionState? {
        try store.getTransactionState(walletId: WalletId.from(id: walletId), transactionId: Primitives.TransactionId(id: transactionId)).map { $0.toGem() }
    }

    public func updateTransactionHash(walletId: String, transactionId: Gemstone.TransactionId, hash: String) async throws {
        try store.updateTransactionHash(
            walletId: WalletId.from(id: walletId),
            transactionId: Primitives.TransactionId(id: transactionId),
            hash: hash,
        )
    }

    public func deleteTransaction(walletId: String, transactionId: Gemstone.TransactionId) async throws {
        try store.deleteTransaction(walletId: WalletId.from(id: walletId), transactionId: Primitives.TransactionId(id: transactionId))
    }

    public func updateTransaction(walletId: String, transactionId: Gemstone.TransactionId, update: GemTransactionStateUpdate) async throws -> Bool {
        try store.updateTransaction(
            walletId: WalletId.from(id: walletId),
            transactionId: Primitives.TransactionId(id: transactionId),
            state: update.state.toPrimitives(),
            fee: update.fee?.description,
            blockNumber: update.blockNumber.flatMap { Int($0) },
            metadata: update.metadata,
            confirmationEtaSeconds: update.confirmationEtaSeconds,
            assetIds: update.assetIds?.map { Primitives.AssetId(core: $0) },
        ) > 0
    }
}
