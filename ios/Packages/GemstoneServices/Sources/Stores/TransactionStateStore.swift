// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.TransactionId
import typealias Gemstone.TransactionState
import protocol Gemstone.GemTransactionStateStore
import struct Gemstone.GemPendingTransaction
import struct Gemstone.GemTransactionStateUpdate
import typealias Gemstone.Transaction
import GemstonePrimitives
import Primitives
import Store

public final class GemstoneTransactionStateStore: GemTransactionStateStore, @unchecked Sendable {
    private let store: TransactionStore

    public init(store: TransactionStore) {
        self.store = store
    }

    public func getPendingTransactions() async throws -> [GemPendingTransaction] {
        try store.getTransactionWallets(states: [.pending, .inTransit]).map { GemPendingTransaction(wallet: $0.wallet.map(), transaction: $0.transaction.json()) }
    }

    public func getTransaction(walletId: String, transactionId: Gemstone.TransactionId) async throws -> GemPendingTransaction? {
        try store.getTransactionWallet(walletId: WalletId.from(id: walletId), transactionId: Primitives.TransactionId(id: transactionId))
            .map { GemPendingTransaction(wallet: $0.wallet.map(), transaction: $0.transaction.json()) }
    }

    public func addTransactions(walletId: String, transactions: [Gemstone.Transaction]) async throws {
        try store.addTransactions(walletId: WalletId.from(id: walletId), transactions: transactions.map { try Primitives.Transaction($0) })
    }

    public func getState(walletId: String, transactionId: Gemstone.TransactionId) async throws -> Gemstone.TransactionState? {
        try store.getTransactionState(walletId: WalletId.from(id: walletId), transactionId: Primitives.TransactionId(id: transactionId)).map { $0.map() }
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
            state: update.state.map(),
            fee: update.fee?.description,
            blockNumber: update.blockNumber.flatMap { Int($0) },
            metadata: update.metadata,
            confirmationEtaSeconds: update.confirmationEtaSeconds,
        ) > 0
    }
}
