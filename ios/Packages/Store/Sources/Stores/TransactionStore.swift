// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct TransactionStore: Sendable {
    let db: DatabaseQueue

    public init(db: DB) {
        self.db = db.dbQueue
    }

    public func getTransactionWallets(
        states: [TransactionState],
    ) throws -> [TransactionWallet] {
        try db.read { db in
            try TransactionRecord
                .including(required: TransactionRecord.wallet.including(all: WalletRecord.accounts))
                .filter(states.map(\.rawValue).contains(TransactionRecord.Columns.state))
                .asRequest(of: WalletTransactionInfo.self)
                .fetchAll(db)
                .map(\.transactionWallet)
        }
    }

    public func getTransactionWallet(
        walletId: WalletId,
        transactionId: TransactionId,
    ) throws -> TransactionWallet? {
        try db.read { db in
            try TransactionRecord
                .including(required: TransactionRecord.wallet.including(all: WalletRecord.accounts))
                .filter(TransactionRecord.Columns.walletId == walletId.id)
                .filter(TransactionRecord.Columns.transactionId == transactionId.identifier)
                .asRequest(of: WalletTransactionInfo.self)
                .fetchOne(db)?
                .transactionWallet
        }
    }

    public func getTransactions(states: [TransactionState]) throws -> [Transaction] {
        try db.read { db in
            try TransactionRecord
                .filter(states.map(\.rawValue).contains(TransactionRecord.Columns.state))
                .fetchAll(db)
                .compactMap { $0.mapToTransaction() }
        }
    }

    func getTransactionAssetAssociations(for transactionId: TransactionId) throws -> [TransactionAssetAssociationRecord] {
        try db.read { db in
            try TransactionAssetAssociationRecord
                .joining(required: TransactionAssetAssociationRecord.transaction.filter(TransactionRecord.Columns.transactionId == transactionId.identifier))
                .fetchAll(db)
        }
    }

    public func getTransaction(walletId: WalletId, transactionId: TransactionId) throws -> TransactionExtended {
        try db.read { db in
            try TransactionRequest(walletId: walletId, transactionId: transactionId).fetch(db)
        }
    }

    public func getSwapHistory(walletId: WalletId) throws -> [TransactionSwapMetadata] {
        try db.read { db in
            try TransactionRecord
                .filter(TransactionRecord.Columns.walletId == walletId.id)
                .filter(TransactionRecord.Columns.type == TransactionType.swap.rawValue)
                .fetchAll(db)
                .compactMap { $0.metadata?.decode(TransactionSwapMetadata.self) }
        }
    }

    public func addTransactions(walletId: WalletId, transactions: [Transaction]) throws {
        if transactions.isEmpty {
            return
        }
        try db.write { db in
            for transaction in transactions {
                let record = try transaction.record(walletId: walletId.id).upsertAndFetch(db, as: TransactionRecord.self)
                if let id = record.id {
                    try TransactionAssetAssociationRecord
                        .filter(TransactionAssetAssociationRecord.Columns.transactionId == id)
                        .deleteAll(db)

                    try transaction.assetIds.forEach {
                        try TransactionAssetAssociationRecord(transactionId: id, assetId: $0).upsert(db)
                    }
                }
            }
        }
    }

    public func getTransactionState(walletId: WalletId, transactionId: TransactionId) throws -> TransactionState? {
        try db.read { db in
            try TransactionRecord
                .filter(TransactionRecord.Columns.walletId == walletId.id)
                .filter(TransactionRecord.Columns.transactionId == transactionId.identifier)
                .fetchOne(db)
                .map(\.state)
        }
    }

    public func renameTransaction(walletId: WalletId, transactionId: TransactionId, newTransactionId: TransactionId) throws {
        try updateValues(walletId: walletId, transactionId: transactionId, values: [
            TransactionRecord.Columns.transactionId.set(to: newTransactionId.identifier),
            TransactionRecord.Columns.hash.set(to: newTransactionId.hash),
        ])
    }

    public func deleteTransaction(walletId: WalletId, transactionId: TransactionId) throws {
        _ = try db.write { db in
            try TransactionRecord
                .filter(TransactionRecord.Columns.walletId == walletId.id)
                .filter(TransactionRecord.Columns.transactionId == transactionId.identifier)
                .deleteAll(db)
        }
    }

    public func updateTransaction(
        walletId: WalletId,
        transactionId: TransactionId,
        state: TransactionState,
        fee: String?,
        blockNumber: Int?,
        metadata: String?,
        confirmationEtaSeconds: UInt32?,
    ) throws -> Int {
        let values: [ColumnAssignment?] = [
            TransactionRecord.Columns.state.set(to: state.rawValue),
            fee.map { TransactionRecord.Columns.fee.set(to: $0) },
            blockNumber.map { TransactionRecord.Columns.blockNumber.set(to: $0) },
            metadata.map { TransactionRecord.Columns.metadata.set(to: $0) },
            confirmationEtaSeconds.map { TransactionRecord.Columns.confirmationEtaSeconds.set(to: $0) },
        ]
        return try updateValues(walletId: walletId, transactionId: transactionId, values: values.compactMap { $0 })
    }

    public func deleteTransactionId(ids: [String]) throws -> Int {
        try db.write { db in
            try TransactionRecord
                .filter(ids.contains(TransactionRecord.Columns.transactionId))
                .deleteAll(db)
        }
    }

    @discardableResult
    private func updateValues(walletId: WalletId, transactionId: TransactionId, values: [ColumnAssignment]) throws -> Int {
        try db.write { db in
            try TransactionRecord
                .filter(TransactionRecord.Columns.walletId == walletId.id)
                .filter(TransactionRecord.Columns.transactionId == transactionId.identifier)
                .updateAll(db, values)
        }
    }

    @discardableResult
    public func clear() throws -> Int {
        try db.write { db in
            try TransactionRecord
                .deleteAll(db)
        }
    }
}
