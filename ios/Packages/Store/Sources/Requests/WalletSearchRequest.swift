// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct WalletSearchResult: Equatable, Sendable {
    public let assets: [AssetData]
    public let perpetuals: [PerpetualData]
    public let lists: [AssetList]

    public init(assets: [AssetData], perpetuals: [PerpetualData], lists: [AssetList]) {
        self.assets = assets
        self.perpetuals = perpetuals
        self.lists = lists
    }

    public static let empty = WalletSearchResult(assets: [], perpetuals: [], lists: [])
}

public struct WalletSearchRequest: DatabaseQueryable, Hashable {
    public var walletId: WalletId
    public var searchBy: String
    public var scope: WalletSearchTag
    public var limit: Int
    public var types: [SearchItemType]

    public init(walletId: WalletId, searchBy: String = "", scope: WalletSearchTag = .all, limit: Int = 5, types: [SearchItemType] = [.asset, .perpetual]) {
        self.walletId = walletId
        self.searchBy = searchBy
        self.scope = scope
        self.limit = limit
        self.types = types
    }

    public func fetch(_ db: Database) throws -> WalletSearchResult {
        let query = searchBy.trim()
        let searchKey = scope.searchKey(query: query)

        let assets = types.contains(.asset) ? try fetchAssets(db, query: query, searchKey: searchKey, scope: scope) : []
        let perpetuals = types.contains(.perpetual) && scope.includesPerpetuals ? try fetchPerpetuals(db, query: query, searchKey: searchKey, scope: scope) : []
        let lists = types.contains(.list) && scope.isAll ? try fetchLists(db, searchKey: searchKey) : []

        return WalletSearchResult(assets: assets, perpetuals: perpetuals, lists: lists)
    }
}

// MARK: - Private

extension WalletSearchRequest {
    private func hasPriority(_ db: Database, searchKey: String, column: Column) throws -> Bool {
        guard searchKey.isNotEmpty else { return false }
        return try SearchRecord
            .filter(SearchRecord.Columns.query == searchKey)
            .filter(column != nil)
            .fetchOne(db) != nil
    }

    private func fetchAssets(_ db: Database, query: String, searchKey: String, scope: WalletSearchTag) throws -> [AssetData] {
        let hasPriority = try hasPriority(db, searchKey: searchKey, column: SearchRecord.Columns.assetId)
        guard hasPriority || scope.isAll else { return [] }

        let balanceAlias = TableAlias(name: BalanceRecord.databaseTableName)
        let priceAlias = TableAlias(name: PriceRecord.databaseTableName)
        let searchAlias = TableAlias(name: SearchRecord.databaseTableName)
        let totalFiatValue = balanceAlias[BalanceRecord.Columns.totalAmount] * (priceAlias[PriceRecord.Columns.price] ?? 0)

        var request = AssetRecord
            .including(optional: AssetRecord.account)
            .including(optional: AssetRecord.balance)
            .including(optional: AssetRecord.price)
            .joining(optional: AssetRecord.balance.filter(BalanceRecord.Columns.walletId == walletId.id))
            .filter(TableAlias(name: AccountRecord.databaseTableName)[AccountRecord.Columns.walletId] == walletId.id)

        if hasPriority {
            request = request
                .joining(required: AssetRecord.search.filter(SearchRecord.Columns.query == searchKey))
                .order(totalFiatValue.desc, searchAlias[SearchRecord.Columns.priority].ascNullsLast, AssetRecord.Columns.rank.desc)
        } else {
            request = request
                .filter(AssetRecord.Columns.symbol.like("%%\(query)%%") || AssetRecord.Columns.name.like("%%\(query)%%") || AssetRecord.Columns.tokenId.like("%%\(query)%%"))
                .order(balanceAlias[BalanceRecord.Columns.isPinned].desc, balanceAlias[BalanceRecord.Columns.isEnabled].desc, totalFiatValue.desc, AssetRecord.Columns.rank.desc)
        }

        return try request.limit(limit).asRequest(of: AssetRecordInfo.self).fetchAll(db).map(\.assetData)
    }

    private func fetchPerpetuals(_ db: Database, query: String, searchKey: String, scope: WalletSearchTag) throws -> [PerpetualData] {
        let hasPriority = try hasPriority(db, searchKey: searchKey, column: SearchRecord.Columns.perpetualId)
        guard hasPriority || scope.isAll else { return [] }

        let searchAlias = TableAlias(name: SearchRecord.databaseTableName)
        let assetAlias = TableAlias(name: AssetRecord.databaseTableName)

        var request = PerpetualRecord.including(required: PerpetualRecord.asset)

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

    private func fetchLists(_ db: Database, searchKey: String) throws -> [AssetList] {
        let searchAlias = TableAlias(name: SearchRecord.databaseTableName)

        return try AssetListRecord
            .joining(required: AssetListRecord.search.filter(SearchRecord.Columns.query == searchKey))
            .order(searchAlias[SearchRecord.Columns.priority].asc)
            .fetchAll(db)
            .map(\.assetList)
    }
}
