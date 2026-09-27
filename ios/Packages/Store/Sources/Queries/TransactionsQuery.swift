// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct TransactionsQuery: DatabaseQueryable {
    private let walletId: WalletId
    private let type: TransactionsQueryType
    private let limit: Int?

    public var filter: TransactionsFilter?

    public init(
        walletId: WalletId,
        type: TransactionsQueryType,
        filter: TransactionsFilter? = nil,
        limit: Int? = nil,
    ) {
        self.walletId = walletId
        self.type = type
        self.filter = filter
        self.limit = limit
    }

    public func fetch(_ db: Database) throws -> [TransactionListItem] {
        try Self.fetch(db, type: type, filter: filter, walletId: walletId, limit: limit)
    }

    public static func fetch(
        _ db: Database,
        type: TransactionsQueryType,
        filter: TransactionsFilter?,
        walletId: WalletId,
        limit: Int? = nil,
    ) throws -> [TransactionListItem] {
        let request = query(walletId: walletId, type: type, filter: filter)
        return try fetch(db, request: limit.map { request.limit($0) } ?? request)
    }

    static func fetch(_ db: Database, request: QueryInterfaceRequest<TransactionRecord>) throws -> [TransactionListItem] {
        try request
            .including(required: TransactionRecord.asset)
            .joining(required: TransactionRecord.feeAsset)
            .including(all: TransactionRecord.assets)
            .including(optional: TransactionRecord.fromAddress)
            .including(optional: TransactionRecord.toAddress)
            .order(TransactionRecord.Columns.date.desc)
            .asRequest(of: TransactionListInfo.self)
            .fetchAll(db)
            .map { $0.mapToTransactionListItem() }
    }

    static func fetchExtended(_ db: Database, request: QueryInterfaceRequest<TransactionRecord>) throws -> [TransactionExtended] {
        try request
            .including(required: TransactionRecord.asset)
            .including(required: TransactionRecord.feeAsset)
            .including(optional: TransactionRecord.price)
            .including(optional: TransactionRecord.feePrice)
            .including(all: TransactionRecord.assets)
            .including(all: TransactionRecord.prices)
            .including(optional: TransactionRecord.fromAddress)
            .including(optional: TransactionRecord.toAddress)
            .order(TransactionRecord.Columns.date.desc)
            .asRequest(of: TransactionInfo.self)
            .fetchAll(db)
            .map { try $0.mapToTransactionExtended() }
    }

    static func query(
        walletId: WalletId,
        type: TransactionsQueryType,
        filter: TransactionsFilter?,
    ) -> QueryInterfaceRequest<TransactionRecord> {
        var request = TransactionRecord
            .filter(TransactionRecord.Columns.walletId == walletId.id)
            .distinct()

        switch type {
        case let .asset(assetId):
            request = request.joining(required: TransactionRecord.assetsAssociation.filter(TransactionAssetAssociationRecord.Columns.assetId == assetId.identifier))
        case let .transaction(id):
            request = request.filter(TransactionRecord.Columns.transactionId == id)
        case .all:
            break
        }

        return filter.map { Self.filtered(request: request, $0) } ?? request
    }
}

// MARK: - Private

extension TransactionsQuery {
    static func filtered(request: QueryInterfaceRequest<TransactionRecord>, _ filter: TransactionsFilter) -> QueryInterfaceRequest<TransactionRecord> {
        var request = request
        if let assetId = filter.assetId {
            request = request.joining(required: TransactionRecord.assetsAssociation.filter(TransactionAssetAssociationRecord.Columns.assetId == assetId.identifier))
        }
        if filter.chains.isNotEmpty {
            request = request.filter(filter.chains.map(\.rawValue).contains(TransactionRecord.Columns.chain))
        }
        if filter.transactionTypes.isNotEmpty {
            request = request.filter(filter.transactionTypes.map(\.rawValue).contains(TransactionRecord.Columns.type))
        }
        if filter.states.isNotEmpty {
            request = request.filter(filter.states.map(\.rawValue).contains(TransactionRecord.Columns.state))
        }
        if let rank = filter.assetRankGreaterThan {
            request = request.joining(required: TransactionRecord.asset.filter(AssetRecord.Columns.rank > Int(rank)))
        }
        return request
    }
}

extension TransactionsQuery: Equatable {}
