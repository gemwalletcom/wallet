import Foundation
import GRDB
import Primitives

public struct AssetsRequest: DatabaseQueryable {
    public static let defaultQueryLimit = 100

    public var walletId: WalletId
    public var searchBy: String
    public var filters: [AssetsRequestFilter]
    public var limit: Int?

    public init(
        walletId: WalletId,
        searchBy: String = "",
        filters: [AssetsRequestFilter] = [],
        limit: Int? = AssetsRequest.defaultQueryLimit,
    ) {
        self.walletId = walletId
        self.searchBy = searchBy
        self.filters = filters
        self.limit = limit
    }

    public func fetch(_ db: Database) throws -> [AssetData] {
        let searchBy = searchBy.trim()

        let filters = if searchBy.isEmpty {
            filters
        } else {
            try filters + [.search(searchBy, hasPriorityAssets: hasPriorityAssets(db, query: searchBy))]
        }

        if filters.contains(.priceAlerts) {
            return try fetchAllAssetRecordsRequest(db, filters: filters)
                .map { $0.mapToEmptyAssetData() }
        }

        return try fetchAssetsSearch(walletId: walletId, filters: filters)
            .fetchAll(db)
            .map(\.assetData)
    }

    static func applyFilters(request: QueryInterfaceRequest<AssetRecord>, _ filters: [AssetsRequestFilter]) -> QueryInterfaceRequest<AssetRecord> {
        var request: QueryInterfaceRequest<AssetRecord> = request
        for filter in filters {
            switch filter {
            case .enabled,
                 .buyable,
                 .sellable,
                 .swappable,
                 .stakeable,
                 .chains,
                 .chainsOrAssets,
                 .search,
                 .enabledBalance,
                 .disabledBalance,
                 .hasBalance,
                 .hasAvailableBalance,
                 .priceAlerts:
                request = Self.applyFilter(request: request, filter)
            }
        }
        return request
    }
}

// MARK: - Private

extension AssetsRequest {
    private func hasPriorityAssets(_ db: Database, query: String) throws -> Bool {
        try SearchRecord
            .filter(SearchRecord.Columns.query == query)
            .filter(SearchRecord.Columns.assetId != nil)
            .limit(1).fetchOne(db) != nil
    }

    private static func applyFilter(request: QueryInterfaceRequest<AssetRecord>, _ filter: AssetsRequestFilter) -> QueryInterfaceRequest<AssetRecord> {
        switch filter {
        case let .search(query, hasPriorityAssets):
            if hasPriorityAssets {
                let totalValue = (TableAlias(name: BalanceRecord.databaseTableName)[BalanceRecord.Columns.totalAmount] * (TableAlias(name: PriceRecord.databaseTableName)[PriceRecord.Columns.price] ?? 0))
                return request.joining(required: AssetRecord.search
                    .filter(SearchRecord.Columns.query == query))
                    .order(
                        totalValue.desc,
                        (totalValue == 0).desc,
                        TableAlias(name: SearchRecord.databaseTableName)[SearchRecord.Columns.priority].ascNullsLast,
                        TableAlias(name: AssetRecord.databaseTableName)[AssetRecord.Columns.rank].desc,
                    )
            }
            return request
                .filter(AssetRecord.textSearchFilter(query: query))
                .order(
                    AssetRecord.Columns.rank.desc,
                )
        case .hasBalance:
            return request
                .filter(
                    TableAlias(name: BalanceRecord.databaseTableName)[BalanceRecord.Columns.totalAmount] > 0,
                )
        case .enabled:
            return request
                .filter(
                    TableAlias(name: AssetRecord.databaseTableName)[AssetRecord.Columns.isEnabled] == true,
                )
        case .hasAvailableBalance:
            return request
                .filter(
                    TableAlias(name: BalanceRecord.databaseTableName)[BalanceRecord.Columns.availableAmount] > 0,
                )
        case .buyable:
            return request
                .filter(
                    TableAlias(name: AssetRecord.databaseTableName)[AssetRecord.Columns.isBuyable] == true,
                )
        case .sellable:
            return request
                .filter(
                    TableAlias(name: AssetRecord.databaseTableName)[AssetRecord.Columns.isSellable] == true,
                )
        case .swappable:
            return request
                .filter(
                    TableAlias(name: AssetRecord.databaseTableName)[AssetRecord.Columns.isSwappable] == true,
                )
        case .stakeable:
            return request
                .filter(
                    TableAlias(name: AssetRecord.databaseTableName)[AssetRecord.Columns.isStakeable] == true,
                )
        case .enabledBalance:
            return request
                .filter(
                    TableAlias(name: BalanceRecord.databaseTableName)[BalanceRecord.Columns.isEnabled] == true,
                )
        case .disabledBalance:
            return request
                .filter(
                    TableAlias(name: BalanceRecord.databaseTableName)[BalanceRecord.Columns.isEnabled] == false,
                )
        case let .chains(chains):
            if chains.isEmpty {
                return request
            }
            return request.filter(chains.contains(AssetRecord.Columns.chain))
        case let .chainsOrAssets(chains, assetIds):
            return request
                .filter(chains.contains(AssetRecord.Columns.chain) || assetIds.contains(AssetRecord.Columns.id))
                .filter(AssetRecord.Columns.isEnabled == true || AssetRecord.Columns.isEnabled == false)
        case .priceAlerts:
            return request
        }
    }

    private func fetchAssetsSearch(
        walletId: WalletId,
        filters: [AssetsRequestFilter],
    ) -> QueryInterfaceRequest<AssetRecordInfo> {
        let totalValue = (TableAlias(name: BalanceRecord.databaseTableName)[BalanceRecord.Columns.totalAmount] * (TableAlias(name: PriceRecord.databaseTableName)[PriceRecord.Columns.price] ?? 0))
        let request = AssetRecord
            .including(optional: AssetRecord.account)
            .including(optional: AssetRecord.balance)
            .including(optional: AssetRecord.price)
            .filter(AssetRecord.Columns.rank >= 0)
            .joining(optional: AssetRecord.balance
                .filter(BalanceRecord.Columns.walletId == walletId.id))
            .filter(
                TableAlias(name: AccountRecord.databaseTableName)[BalanceRecord.Columns.walletId] == walletId.id,
            )
            .order(
                TableAlias(name: BalanceRecord.databaseTableName)[BalanceRecord.Columns.isPinned].desc,
                TableAlias(name: BalanceRecord.databaseTableName)[BalanceRecord.Columns.isEnabled].desc,
                totalValue.desc,
                (totalValue == 0).desc,
                AssetRecord.Columns.rank.desc,
            )

        return Self.applyFilters(request: limit.map { request.limit($0) } ?? request, filters)
            .asRequest(of: AssetRecordInfo.self)
    }
}

/// Specific case for the price alerts scene:
/// This is necessary because watch-only wallets do not create accounts for other networks.
/// On the price alerts screen, we fetch all assets and fill them with empty data.
extension AssetsRequest {
    private func fetchAllAssetRecordsRequest(
        _ db: Database,
        filters: [AssetsRequestFilter],
    ) throws -> [PriceAlertAssetRecordInfo] {
        var request = AssetRecord
            .including(all: AssetRecord.priceAlerts)
            .including(optional: AssetRecord.price)
            .filter(AssetRecord.Columns.rank >= 0)
            .order(AssetRecord.Columns.rank.desc)
            .limit(Self.defaultQueryLimit)

        request = Self.applyFilters(request: request, filters)

        return try request
            .asRequest(of: PriceAlertAssetRecordInfo.self)
            .fetchAll(db)
    }
}

extension AssetsRequest: Equatable {}
