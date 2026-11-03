// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct WalletSearchResult: Equatable, Sendable {
    public let assets: [AssetData]
    public let perpetuals: [PerpetualData]
    public let collections: [NFTData]
    public let lists: [AssetList]

    public init(assets: [AssetData], perpetuals: [PerpetualData], collections: [NFTData], lists: [AssetList]) {
        self.assets = assets
        self.perpetuals = perpetuals
        self.collections = collections
        self.lists = lists
    }

    public static let empty = WalletSearchResult(assets: [], perpetuals: [], collections: [], lists: [])
}

public struct WalletSearchQuery: DatabaseQueryable, Hashable {
    public var walletId: WalletId
    public var searchBy: String
    public var searchKey: String
    public var scope: WalletSearchTag
    public var limit: Int
    public var types: [SearchItemType]

    public init(walletId: WalletId, searchBy: String = "", searchKey: String = "", scope: WalletSearchTag = .all, limit: Int = 5, types: [SearchItemType] = [.asset, .perpetual]) {
        self.walletId = walletId
        self.searchBy = searchBy
        self.searchKey = searchKey
        self.scope = scope
        self.limit = limit
        self.types = types
    }

    public func fetch(_ db: Database) throws -> WalletSearchResult {
        let query = searchBy.trim()

        let assets = types.contains(.asset) ? try loadAssets(db, searchKey: searchKey, scope: scope) : []
        let perpetuals = types.contains(.perpetual) ? try loadPerpetuals(db, query: query, searchKey: searchKey, scope: scope) : []
        let collections = types.contains(.nft) && scope.isAll && query.isNotEmpty ? try NFTQuery(walletId: walletId, filter: .all).fetch(db) : []
        let lists = types.contains(.list) && scope.isAll ? try loadLists(db, searchKey: searchKey) : []

        return WalletSearchResult(assets: assets, perpetuals: perpetuals, collections: collections, lists: lists)
    }
}

// MARK: - Private

extension WalletSearchQuery {
    private func hasPriority(_ db: Database, searchKey: String, column: Column) throws -> Bool {
        guard searchKey.isNotEmpty else { return false }
        return try SearchRecord
            .filter(SearchRecord.Columns.query == searchKey)
            .filter(column != nil)
            .fetchOne(db) != nil
    }

    private func loadAssets(_ db: Database, searchKey: String, scope: WalletSearchTag) throws -> [AssetData] {
        switch scope {
        case .all:
            try AssetsQuery(walletId: walletId, searchBy: searchBy, limit: limit).fetch(db)
        case .list:
            try AssetsQuery.prioritized(request: AssetsQuery.walletAssets(walletId: walletId), searchKey: searchKey)
                .limit(limit)
                .asRequest(of: AssetRecordInfo.self)
                .fetchAll(db)
                .map(\.assetData)
        }
    }

    private func loadPerpetuals(_ db: Database, query: String, searchKey: String, scope: WalletSearchTag) throws -> [PerpetualData] {
        let hasPriority = try hasPriority(db, searchKey: searchKey, column: SearchRecord.Columns.perpetualId)
        guard hasPriority || scope.isAll else { return [] }

        let searchAlias = TableAlias(name: SearchRecord.databaseTableName)
        let assetAlias = TableAlias(name: AssetRecord.databaseTableName)

        var request = PerpetualRecord
            .filter(PerpetualRecord.Columns.isEnabled)
            .including(required: PerpetualRecord.asset)

        if hasPriority {
            request = request
                .joining(required: PerpetualRecord.search.filter(SearchRecord.Columns.query == searchKey))
                .order(searchAlias[SearchRecord.Columns.priority].ascNullsLast, PerpetualRecord.Columns.volume24h.desc)
        } else {
            request = request
                .filter(PerpetualRecord.Columns.name.like("%%\(query)%%") || assetAlias[AssetRecord.Columns.symbol].like("%%\(query)%%"))
                .order(PerpetualRecord.Columns.volume24h.desc)
        }

        return try request.limit(limit).asRequest(of: PerpetualInfo.self).fetchAll(db).map { $0.mapToPerpetualData() }
    }

    private func loadLists(_ db: Database, searchKey: String) throws -> [AssetList] {
        let searchAlias = TableAlias(name: SearchRecord.databaseTableName)

        return try AssetListRecord
            .joining(required: AssetListRecord.search.filter(SearchRecord.Columns.query == searchKey))
            .order(searchAlias[SearchRecord.Columns.priority].asc)
            .fetchAll(db)
            .map(\.assetList)
    }
}
