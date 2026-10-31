import Foundation
import GRDB
import Primitives

public struct AssetsQuery: DatabaseQueryable {
    public var walletId: WalletId
    public var scope: AssetsQueryScope
    public var searchBy: String
    public var filters: [AssetsQueryFilter]
    public var limit: Int?

    public init(
        walletId: WalletId,
        scope: AssetsQueryScope = .wallet,
        searchBy: String = "",
        filters: [AssetsQueryFilter] = [],
        limit: Int? = nil,
    ) {
        self.walletId = walletId
        self.scope = scope
        self.searchBy = searchBy
        self.filters = filters
        self.limit = limit
    }

    public func fetch(_ db: Database) throws -> [AssetData] {
        switch scope {
        case .wallet:
            try filtered(db, request: Self.walletAssets(walletId: walletId))
                .asRequest(of: AssetRecordInfo.self)
                .fetchAll(db)
                .map(\.assetData)
        case .allAssets:
            try allAssetRecords(db)
                .map { $0.mapToEmptyAssetData() }
        }
    }

    static func filtered(request: QueryInterfaceRequest<AssetRecord>, _ filters: [AssetsQueryFilter]) -> QueryInterfaceRequest<AssetRecord> {
        filters.reduce(request) { Self.filtered(request: $0, $1) }
    }

    static func walletAssets(walletId: WalletId) -> QueryInterfaceRequest<AssetRecord> {
        ordered(request: AssetRecord
            .including(optional: AssetRecord.account)
            .including(optional: AssetRecord.balance)
            .including(optional: AssetRecord.price)
            .filter(AssetRecord.Columns.rank >= 0)
            .joining(optional: AssetRecord.balance
                .filter(BalanceRecord.Columns.walletId == walletId.id))
            .filter(
                TableAlias(name: AccountRecord.databaseTableName)[BalanceRecord.Columns.walletId] == walletId.id,
            ))
    }

    static func prioritized(request: QueryInterfaceRequest<AssetRecord>, searchKey: String) -> QueryInterfaceRequest<AssetRecord> {
        request.joining(required: AssetRecord.search
            .filter(SearchRecord.Columns.query == searchKey))
            .order(
                AssetRecord.fiatValue.desc,
                TableAlias(name: SearchRecord.databaseTableName)[SearchRecord.Columns.priority].ascNullsLast,
                TableAlias(name: AssetRecord.databaseTableName)[AssetRecord.Columns.rank].desc,
            )
    }
}

// MARK: - Private

extension AssetsQuery {
    private func hasPriorityAssets(_ db: Database, query: String) throws -> Bool {
        try SearchRecord
            .filter(SearchRecord.Columns.query == query)
            .filter(SearchRecord.Columns.assetId != nil)
            .fetchOne(db) != nil
    }

    private func filtered(_ db: Database, request: QueryInterfaceRequest<AssetRecord>) throws -> QueryInterfaceRequest<AssetRecord> {
        let request = Self.filtered(request: limit.map { request.limit($0) } ?? request, filters)
        let query = searchBy.trim()
        guard query.isNotEmpty else { return request }
        return try hasPriorityAssets(db, query: query)
            ? Self.prioritized(request: request, searchKey: query)
            : request.filter(AssetRecord.textSearchFilter(query: query))
    }

    private static func filtered(request: QueryInterfaceRequest<AssetRecord>, _ filter: AssetsQueryFilter) -> QueryInterfaceRequest<AssetRecord> {
        switch filter {
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
        }
    }

    private static func ordered(request: QueryInterfaceRequest<AssetRecord>) -> QueryInterfaceRequest<AssetRecord> {
        request.order(
            TableAlias(name: BalanceRecord.databaseTableName)[BalanceRecord.Columns.isPinned].desc,
            TableAlias(name: BalanceRecord.databaseTableName)[BalanceRecord.Columns.isEnabled].desc,
            AssetRecord.fiatValue.desc,
            AssetRecord.Columns.rank.desc,
        )
    }
}

extension AssetsQuery {
    private func allAssetRecords(_ db: Database) throws -> [PriceAlertAssetRecordInfo] {
        let request = Self.ordered(request: AssetRecord
            .including(all: AssetRecord.priceAlerts)
            .including(optional: AssetRecord.price)
            .including(optional: AssetRecord.balance.filter(BalanceRecord.Columns.walletId == walletId.id))
            .filter(AssetRecord.Columns.rank >= 0))

        return try filtered(db, request: request)
            .asRequest(of: PriceAlertAssetRecordInfo.self)
            .fetchAll(db)
    }
}

extension AssetsQuery: Equatable {}

extension AssetsQueryFilter {
    var referencesBalances: Bool {
        switch self {
        case .hasBalance,
             .hasAvailableBalance,
             .enabledBalance,
             .disabledBalance:
            true
        case .enabled,
             .buyable,
             .sellable,
             .swappable,
             .chains,
             .chainsOrAssets:
            false
        }
    }
}
