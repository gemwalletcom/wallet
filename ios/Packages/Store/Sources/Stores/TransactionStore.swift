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
            guard let transaction = try TransactionsRequest.fetch(
                db, type: .transaction(id: transactionId.identifier), filters: [], walletId: walletId,
            ).first else {
                throw RecordError.recordNotFound(databaseTableName: TransactionRecord.databaseTableName, key: [:])
            }
            return transaction
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
                try updateAssetAssociations(db, record: record)
            }
        }
    }

    public func getTransactionState(walletId: WalletId, transactionId: TransactionId) throws -> TransactionState? {
        try db.read { db in
            try TransactionRecord
                .select(TransactionRecord.Columns.state, as: String.self)
                .filter(TransactionRecord.Columns.walletId == walletId.id)
                .filter(TransactionRecord.Columns.transactionId == transactionId.identifier)
                .fetchOne(db)
                .flatMap(TransactionState.init(rawValue:))
        }
    }

    public func updateTransactionHash(walletId: WalletId, transactionId: TransactionId, hash: String) throws {
        guard transactionId.hash != hash else { return }
        let newTransactionId = TransactionId(chain: transactionId.chain, hash: hash)
        try db.write { db in
            let transactions = TransactionRecord.filter(TransactionRecord.Columns.walletId == walletId.id)
            let source = transactions.filter(TransactionRecord.Columns.transactionId == transactionId.identifier)
            guard let sourceId = try source.select(TransactionRecord.Columns.id, as: Int.self).fetchOne(db) else {
                return
            }
            let target = transactions.filter(TransactionRecord.Columns.transactionId == newTransactionId.identifier)
            if let targetId = try target.select(TransactionRecord.Columns.id, as: Int.self).fetchOne(db) {
                try source.deleteAll(db)
                try TransactionAssetAssociationRecord.filter(TransactionAssetAssociationRecord.Columns.transactionId == targetId).deleteAll(db)
                try target.updateAndFetchAll(db, [TransactionRecord.Columns.id.set(to: sourceId)])
                    .forEach { try updateAssetAssociations(db, record: $0) }
            } else {
                try source.updateAll(db, [
                    TransactionRecord.Columns.transactionId.set(to: newTransactionId.identifier),
                    TransactionRecord.Columns.hash.set(to: hash),
                ])
            }
        }
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
        return try db.write { db in
            let request = TransactionRecord
                .filter(TransactionRecord.Columns.walletId == walletId.id)
                .filter(TransactionRecord.Columns.transactionId == transactionId.identifier)
            let updated = try request.updateAll(db, values.compactMap { $0 })
            if updated > 0, metadata != nil, let record = try request.fetchOne(db) {
                try updateAssetAssociations(db, record: record)
            }
            return updated
        }
    }

    public func deleteTransactionId(ids: [String]) throws -> Int {
        try db.write { db in
            try TransactionRecord
                .filter(ids.contains(TransactionRecord.Columns.transactionId))
                .deleteAll(db)
        }
    }

    private func updateAssetAssociations(_ db: Database, record: TransactionRecord) throws {
        guard let id = record.id else {
            return
        }
        let assetIds = record.mapToTransaction().assetIds
        let storedIds = try AssetRecord
            .select(AssetRecord.Columns.id, as: String.self)
            .filter(assetIds.map(\.identifier).contains(AssetRecord.Columns.id))
            .fetchSet(db)
        try TransactionAssetAssociationRecord
            .filter(TransactionAssetAssociationRecord.Columns.transactionId == id)
            .deleteAll(db)
        try assetIds
            .filter { storedIds.contains($0.identifier) }
            .forEach { try TransactionAssetAssociationRecord(transactionId: id, assetId: $0).upsert(db) }
    }

    @discardableResult
    public func clear() throws -> Int {
        try db.write { db in
            try TransactionRecord
                .deleteAll(db)
        }
    }
}
