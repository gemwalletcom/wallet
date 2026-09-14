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

public struct WalletSearchRequest: DatabaseQueryable, Hashable {
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

        let assets = types.contains(.asset) ? try loadAssets(db, query: query, searchKey: searchKey, scope: scope) : []
        let perpetuals = types.contains(.perpetual) ? try loadPerpetuals(db, query: query, searchKey: searchKey, scope: scope) : []
        let collections = types.contains(.nft) && scope.isAll && query.isNotEmpty ? try NFTRequest(walletId: walletId, filter: .all).fetch(db) : []
        let lists = types.contains(.list) && scope.isAll ? try loadLists(db, searchKey: searchKey) : []

        return WalletSearchResult(assets: assets, perpetuals: perpetuals, collections: collections, lists: lists)
    }
}

// MARK: - Private

extension WalletSearchRequest {
    private func loadAssets(_ db: Database, query: String, searchKey: String, scope: WalletSearchTag) throws -> [AssetData] {
        let balanceAlias = TableAlias(name: BalanceRecord.databaseTableName)
        let priceAlias = TableAlias(name: PriceRecord.databaseTableName)
        let searchAlias = TableAlias(name: SearchRecord.databaseTableName)
        let totalFiatValue = balanceAlias[BalanceRecord.Columns.totalAmount] * (priceAlias[PriceRecord.Columns.price] ?? 0)
        let matchesQuery = scope.isAll ? AssetRecord.textSearchFilter(query: query) : false.sqlExpression

        let request = AssetRecord
            .including(optional: AssetRecord.account)
            .including(optional: AssetRecord.balance)
            .including(optional: AssetRecord.price)
            .filter(AssetRecord.Columns.rank >= 0)
            .joining(optional: AssetRecord.balance.filter(BalanceRecord.Columns.walletId == walletId.id))
            .filter(TableAlias(name: AccountRecord.databaseTableName)[AccountRecord.Columns.walletId] == walletId.id)
            .joining(optional: AssetRecord.search.filter(SearchRecord.Columns.query == searchKey))
            .filter(matchesQuery || searchAlias[SearchRecord.Columns.priority] != nil)
            .order(
                matchesQuery.desc,
                balanceAlias[BalanceRecord.Columns.isPinned].desc,
                balanceAlias[BalanceRecord.Columns.isEnabled].desc,
                totalFiatValue.desc,
                searchAlias[SearchRecord.Columns.priority].ascNullsLast,
                AssetRecord.Columns.rank.desc,
            )

        return try request.limit(limit).asRequest(of: AssetRecordInfo.self).fetchAll(db).map(\.assetData)
    }

    private func loadPerpetuals(_ db: Database, query: String, searchKey: String, scope: WalletSearchTag) throws -> [PerpetualData] {
        let searchAlias = TableAlias(name: SearchRecord.databaseTableName)
        let assetAlias = TableAlias(name: AssetRecord.databaseTableName)
        let matchesQuery = scope.isAll
            ? PerpetualRecord.Columns.name.like("%%\(query)%%") || assetAlias[AssetRecord.Columns.symbol].like("%%\(query)%%")
            : false.sqlExpression

        let request = PerpetualRecord
            .including(required: PerpetualRecord.asset)
            .joining(optional: PerpetualRecord.search.filter(SearchRecord.Columns.query == searchKey))
            .filter(matchesQuery || searchAlias[SearchRecord.Columns.priority] != nil)
            .order(
                matchesQuery.desc,
                searchAlias[SearchRecord.Columns.priority].ascNullsLast,
                PerpetualRecord.Columns.volume24h.desc,
            )

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
