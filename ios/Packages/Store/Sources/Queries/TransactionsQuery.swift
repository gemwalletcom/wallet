// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct TransactionsQuery: DatabaseQueryable {
    private let walletId: WalletId
    private let type: TransactionsQueryType
    private let limit: Int?

    public var filters: [TransactionsQueryFilter] = []

    public init(
        walletId: WalletId,
        type: TransactionsQueryType,
        filters: [TransactionsQueryFilter] = [],
        limit: Int? = nil,
    ) {
        self.walletId = walletId
        self.type = type
        self.filters = filters
        self.limit = limit
    }

    public func fetch(_ db: Database) throws -> [TransactionListItem] {
        try Self.fetch(db, type: type, filters: filters, walletId: walletId, limit: limit)
    }

    public static func fetch(
        _ db: Database,
        type: TransactionsQueryType,
        filters: [TransactionsQueryFilter],
        walletId: WalletId,
        limit: Int? = nil,
    ) throws -> [TransactionListItem] {
        let request = query(walletId: walletId, type: type, filters: filters)
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
        filters: [TransactionsQueryFilter],
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

        for filter in filters {
            request = Self.filtered(request: request, filter)
        }

        return request
    }
}

// MARK: - Private

extension TransactionsQuery {
    static func filtered(request: QueryInterfaceRequest<TransactionRecord>, _ filter: TransactionsQueryFilter) -> QueryInterfaceRequest<TransactionRecord> {
        switch filter {
        case let .chains(chains):
            guard !chains.isEmpty else { return request }
            return request.filter(chains.contains(TransactionRecord.Columns.chain))
        case let .types(types):
            guard !types.isEmpty else { return request }
            return request.filter(types.contains(TransactionRecord.Columns.type))
        case let .assetRankGreaterThan(rank):
            return request.joining(required: TransactionRecord.asset.filter(AssetRecord.Columns.rank > rank))
        case let .states(states):
            return request.filter(states.contains(TransactionRecord.Columns.state))
        }
    }
}

extension TransactionsQuery: Equatable {}
