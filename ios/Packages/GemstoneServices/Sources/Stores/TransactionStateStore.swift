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
            transactions.map { GemPendingTransaction(walletId: walletId, transaction: $0.toGem()) }
        }
    }

    public func getTransaction(walletId: WalletId, transactionId: Gemstone.TransactionId) async throws -> GemPendingTransaction? {
        guard try store.getTransactionState(walletId: walletId, transactionId: transactionId) != nil else { return nil }
        let transaction = try store.getTransaction(walletId: walletId, transactionId: transactionId).transaction
        return GemPendingTransaction(walletId: walletId, transaction: transaction.toGem())
    }

    public func addTransactions(walletId: WalletId, transactions: [Gemstone.Transaction]) async throws {
        try store.addTransactions(walletId: walletId, transactions: transactions.map(\.transactionAssets))
    }

    public func getState(walletId: WalletId, transactionId: Gemstone.TransactionId) async throws -> Gemstone.TransactionState? {
        try store.getTransactionState(walletId: walletId, transactionId: transactionId).map { $0.toGem() }
    }

    public func updateTransactionHash(walletId: WalletId, transactionId: Gemstone.TransactionId, hash: String) async throws {
        try store.updateTransactionHash(
            walletId: walletId,
            transactionId: transactionId,
            hash: hash,
        )
    }

    public func deleteTransaction(walletId: WalletId, transactionId: Gemstone.TransactionId) async throws {
        try store.deleteTransaction(walletId: walletId, transactionId: transactionId)
    }

    public func updateTransaction(walletId: WalletId, transactionId: Gemstone.TransactionId, update: GemTransactionStateUpdate) async throws -> Bool {
        try store.updateTransaction(
            walletId: walletId,
            transactionId: transactionId,
            state: update.state.toPrimitives(),
            fee: update.fee?.description,
            blockNumber: update.blockNumber.flatMap { Int($0) },
            metadata: update.metadata,
            confirmationEtaSeconds: update.confirmationEtaSeconds,
            assetIds: update.assetIds,
        ) > 0
    }
}
