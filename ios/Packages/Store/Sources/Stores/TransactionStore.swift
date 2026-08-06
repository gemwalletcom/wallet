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
        guard transactions.isNotEmpty else { return }
        try db.write { db in
            for transaction in transactions {
                let record = try transaction.record(walletId: walletId.id).upsertAndFetch(db, as: TransactionRecord.self)
                try replaceAssetAssociations(of: record, with: transaction.assetIds, db: db)
            }
        }
    }

    public func syncTransactions(walletId: WalletId, transactions: [Transaction]) throws {
        guard transactions.isNotEmpty else { return }
        try db.write { db in
            for transaction in transactions {
                let incomingRecord = try transaction.record(walletId: walletId.id)
                guard let storedRecord = try storedRecord(of: incomingRecord, db: db) else {
                    let insertedRecord = try incomingRecord.insertAndFetch(db, as: TransactionRecord.self)
                    try replaceAssetAssociations(of: insertedRecord, with: transaction.assetIds, db: db)
                    continue
                }
                let storedState = storedRecord.transactionState
                let syncedState = storedState.next(onChain: incomingRecord.transactionState)

                try refreshObservedValues(of: incomingRecord, db: db)
                if syncedState != storedState {
                    try refreshState(of: storedRecord, to: syncedState, db: db)
                }
                if syncedState.isCompleted, incomingRecord.metadata != nil {
                    try refreshDescription(of: incomingRecord, db: db)
                    try replaceAssetAssociations(of: storedRecord, with: transaction.assetIds, db: db)
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

    public func updateTransactionId(oldTransactionId: TransactionId, transactionId: TransactionId, hash: String) throws {
        try db.write { db in
            guard let trackedRecord = try TransactionRecord
                .filter(TransactionRecord.Columns.transactionId == oldTransactionId.identifier)
                .fetchOne(db)
            else {
                return
            }

            try deleteIndexedDuplicate(walletId: trackedRecord.walletId, transactionId: transactionId, db: db)

            try TransactionRecord
                .filter(TransactionRecord.Columns.id == trackedRecord.id)
                .updateAll(db, [
                    TransactionRecord.Columns.transactionId.set(to: transactionId.identifier),
                    TransactionRecord.Columns.hash.set(to: hash),
                ])
        }
    }

    public func deleteTransactionId(ids: [String]) throws -> Int {
        try db.write { db in
            try TransactionRecord
                .filter(ids.contains(TransactionRecord.Columns.transactionId))
                .deleteAll(db)
        }
    }

    private func deleteIndexedDuplicate(walletId: String, transactionId: TransactionId, db: Database) throws {
        _ = try TransactionRecord
            .filter(TransactionRecord.Columns.walletId == walletId)
            .filter(TransactionRecord.Columns.transactionId == transactionId.identifier)
            .deleteAll(db)
    }

    private func replaceAssetAssociations(of record: TransactionRecord, with assetIds: [AssetId], db: Database) throws {
        guard let recordId = record.id else { return }
        try TransactionAssetAssociationRecord
            .filter(TransactionAssetAssociationRecord.Columns.transactionId == recordId)
            .deleteAll(db)

        try assetIds.forEach {
            try TransactionAssetAssociationRecord(transactionId: recordId, assetId: $0).upsert(db)
        }
    }

    private func storedRecord(of record: TransactionRecord, db: Database) throws -> TransactionRecord? {
        try TransactionRecord
            .filter(TransactionRecord.Columns.walletId == record.walletId)
            .filter(TransactionRecord.Columns.transactionId == record.transactionId)
            .fetchOne(db)
    }

    private func refreshState(of record: TransactionRecord, to state: TransactionState, db: Database) throws {
        _ = try TransactionRecord
            .filter(TransactionRecord.Columns.walletId == record.walletId)
            .filter(TransactionRecord.Columns.transactionId == record.transactionId)
            .updateAll(db, [TransactionRecord.Columns.state.set(to: state.rawValue)])
    }

    private func refreshDescription(of record: TransactionRecord, db: Database) throws {
        _ = try TransactionRecord
            .filter(TransactionRecord.Columns.walletId == record.walletId)
            .filter(TransactionRecord.Columns.transactionId == record.transactionId)
            .updateAll(db, [
                TransactionRecord.Columns.type.set(to: record.type.rawValue),
                TransactionRecord.Columns.metadata.set(to: try JSONEncoder().encode(record.metadata).encodeString()),
            ])
    }

    private func refreshObservedValues(of record: TransactionRecord, db: Database) throws {
        _ = try TransactionRecord
            .filter(TransactionRecord.Columns.walletId == record.walletId)
            .filter(TransactionRecord.Columns.transactionId == record.transactionId)
            .updateAll(db, [
                TransactionRecord.Columns.from.set(to: record.from),
                TransactionRecord.Columns.to.set(to: record.to),
                TransactionRecord.Columns.contract.set(to: record.contract),
                TransactionRecord.Columns.blockNumber.set(to: record.blockNumber),
                TransactionRecord.Columns.sequence.set(to: record.sequence),
                TransactionRecord.Columns.value.set(to: record.value),
                TransactionRecord.Columns.fee.set(to: record.fee),
                TransactionRecord.Columns.memo.set(to: record.memo),
                TransactionRecord.Columns.updatedAt.set(to: record.updatedAt),
            ])
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
