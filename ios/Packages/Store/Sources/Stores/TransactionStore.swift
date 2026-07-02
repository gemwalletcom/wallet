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
                .filter(states.map(\.rawValue).contains(TransactionRecord.Columns.state) )
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

    public func updateState(id: TransactionId, state: TransactionState) throws -> Int {
        try updateValues(id: id, values: [TransactionRecord.Columns.state.set(to: state.rawValue)])
    }

    public func updateNetworkFee(transactionId: TransactionId, networkFee: String) throws -> Int {
        try updateValues(id: transactionId, values: [TransactionRecord.Columns.fee.set(to: networkFee)])
    }

    public func updateBlockNumber(transactionId: TransactionId, block: Int) throws -> Int {
        try updateValues(id: transactionId, values: [TransactionRecord.Columns.blockNumber.set(to: block)])
    }

    public func updateCreatedAt(transactionId: TransactionId, date: Date) throws -> Int {
        try updateValues(id: transactionId, values: [TransactionRecord.Columns.createdAt.set(to: date)])
    }

    public func updateMetadata(transactionId: TransactionId, metadata: AnyCodableValue) throws -> Int {
        let string = try JSONEncoder().encode(metadata).encodeString()
        return try updateValues(
            id: transactionId,
            values: [TransactionRecord.Columns.metadata.set(to: string)],
        )
    }

    public func updateTransactionId(oldTransactionId: TransactionId, transactionId: TransactionId, hash: String) throws -> TransactionState? {
        try db.write { db in
            let oldRecord = try TransactionRecord
                .filter(TransactionRecord.Columns.transactionId == oldTransactionId.identifier)
                .fetchOne(db)

            if let oldRecord {
                let existingRecord = try TransactionRecord
                    .filter(TransactionRecord.Columns.walletId == oldRecord.walletId)
                    .filter(TransactionRecord.Columns.transactionId == transactionId.identifier)
                    .fetchOne(db)
                if let existingRecord {
                    _ = try TransactionRecord
                        .filter(TransactionRecord.Columns.id == oldRecord.id)
                        .deleteAll(db)
                    return TransactionState(rawValue: existingRecord.state)
                }
            }

            try TransactionRecord
                .filter(TransactionRecord.Columns.transactionId == oldTransactionId.identifier)
                .updateAll(db, [
                    TransactionRecord.Columns.transactionId.set(to: transactionId.identifier),
                    TransactionRecord.Columns.hash.set(to: hash),
                ])
            return nil
        }
    }

    public func deleteTransactionId(ids: [String]) throws -> Int {
        try db.write { db in
            try TransactionRecord
                .filter(ids.contains(TransactionRecord.Columns.transactionId))
                .deleteAll(db)
        }
    }

    private func updateValues(id: TransactionId, values: [ColumnAssignment]) throws -> Int {
        try db.write { db in
            try TransactionRecord
                .filter(TransactionRecord.Columns.transactionId == id.identifier)
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
