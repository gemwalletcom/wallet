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
        try store.getTransactionWallets(states: [.pending, .inTransit]).map { GemPendingTransaction(wallet: $0.wallet.json(), transaction: $0.transaction.json()) }
    }

    public func getTransaction(walletId: String, transactionId: Gemstone.TransactionId) async throws -> GemPendingTransaction? {
        try store.getTransactionWallet(walletId: WalletId.from(id: walletId), transactionId: Primitives.TransactionId(transactionId))
            .map { GemPendingTransaction(wallet: $0.wallet.json(), transaction: $0.transaction.json()) }
    }

    public func addTransactions(walletId: String, transactions: [Gemstone.Transaction]) async throws {
        try store.addTransactions(walletId: WalletId.from(id: walletId), transactions: transactions.map { try Primitives.Transaction($0) })
    }

    public func getState(walletId: String, transactionId: Gemstone.TransactionId) async throws -> Gemstone.TransactionState? {
        try store.getTransactionState(walletId: WalletId.from(id: walletId), transactionId: Primitives.TransactionId(transactionId)).map { $0.json() }
    }

    public func renameTransaction(walletId: String, transactionId: Gemstone.TransactionId, newTransactionId: Gemstone.TransactionId) async throws {
        try store.renameTransaction(
            walletId: WalletId.from(id: walletId),
            transactionId: Primitives.TransactionId(transactionId),
            newTransactionId: Primitives.TransactionId(newTransactionId),
        )
    }

    public func deleteTransaction(walletId: String, transactionId: Gemstone.TransactionId) async throws {
        try store.deleteTransaction(walletId: WalletId.from(id: walletId), transactionId: Primitives.TransactionId(transactionId))
    }

    public func updateTransaction(walletId: String, transactionId: Gemstone.TransactionId, update: GemTransactionStateUpdate) async throws -> Bool {
        try store.updateTransaction(
            walletId: WalletId.from(id: walletId),
            transactionId: Primitives.TransactionId(transactionId),
            state: Primitives.TransactionState(update.state),
            fee: update.fee,
            blockNumber: update.blockNumber.flatMap { Int($0) },
            metadata: update.metadata,
            confirmationEtaSeconds: update.confirmationEtaSeconds,
        ) > 0
    }
}
