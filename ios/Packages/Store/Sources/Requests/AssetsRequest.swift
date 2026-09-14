import Foundation
import GRDB
import Primitives

public struct AssetsRequest: DatabaseQueryable {
    public static let defaultQueryLimit = 100

    public var walletId: WalletId
    public var scope: AssetsRequestScope
    public var searchBy: String
    public var filters: [AssetsRequestFilter]
    public var limit: Int?

    public init(
        walletId: WalletId,
        scope: AssetsRequestScope = .wallet,
        searchBy: String = "",
        filters: [AssetsRequestFilter] = [],
        limit: Int? = AssetsRequest.defaultQueryLimit,
    ) {
        self.walletId = walletId
        self.scope = scope
        self.searchBy = searchBy
        self.filters = filters
        self.limit = limit
    }

    public func fetch(_ db: Database) throws -> [AssetData] {
        let searchBy = searchBy.trim()

        let filters = if searchBy.isEmpty {
            filters
        } else {
            filters + [.search(searchBy)]
        }

        switch scope {
        case .wallet:
            return try loadAssetsSearch(walletId: walletId, filters: filters)
                .fetchAll(db)
                .map(\.assetData)
        case .allAssets:
            return try fetchAllAssetRecordsRequest(db, filters: filters)
                .map { $0.mapToEmptyAssetData() }
        }
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
                 .hasAvailableBalance:
                request = Self.applyFilter(request: request, filter)
            }
        }
        return request
    }
}

// MARK: - Private

extension AssetsRequest {
    private static func applyFilter(request: QueryInterfaceRequest<AssetRecord>, _ filter: AssetsRequestFilter) -> QueryInterfaceRequest<AssetRecord> {
        switch filter {
        case let .search(query):
            let assetAlias = TableAlias(name: AssetRecord.databaseTableName)
            let balanceAlias = TableAlias(name: BalanceRecord.databaseTableName)
            let priceAlias = TableAlias(name: PriceRecord.databaseTableName)
            let searchAlias = TableAlias(name: SearchRecord.databaseTableName)
            let totalValue = balanceAlias[BalanceRecord.Columns.totalAmount] * (priceAlias[PriceRecord.Columns.price] ?? 0)
            let matchesQuery = AssetRecord.textSearchFilter(query: query)

            return request
                .joining(optional: AssetRecord.search.filter(SearchRecord.Columns.query == query))
                .filter(matchesQuery || searchAlias[SearchRecord.Columns.priority] != nil)
                .order(
                    matchesQuery.desc,
                    balanceAlias[BalanceRecord.Columns.isPinned].desc,
                    balanceAlias[BalanceRecord.Columns.isEnabled].desc,
                    totalValue.desc,
                    searchAlias[SearchRecord.Columns.priority].ascNullsLast,
                    assetAlias[AssetRecord.Columns.rank].desc,
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
        }
    }

    private func loadAssetsSearch(
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

extension AssetsRequestFilter {
    var referencesBalances: Bool {
        switch self {
        case .search,
             .hasBalance,
             .hasAvailableBalance,
             .enabledBalance,
             .disabledBalance:
            true
        case .enabled,
             .buyable,
             .sellable,
             .swappable,
             .stakeable,
             .chains,
             .chainsOrAssets:
            false
        }
    }
}
