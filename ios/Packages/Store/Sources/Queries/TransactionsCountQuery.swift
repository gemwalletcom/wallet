// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct TransactionsCountQuery: DatabaseQueryable {
    public var walletId: WalletId
    private let type: TransactionsQueryType
    private let filters: [TransactionsQueryFilter]

    public init(
        walletId: WalletId,
        type: TransactionsQueryType,
        filters: [TransactionsQueryFilter] = [],
    ) {
        self.walletId = walletId
        self.type = type
        self.filters = filters
    }

    public func fetch(_ db: Database) throws -> Int {
        try TransactionsQuery.query(walletId: walletId, type: type, filters: filters)
            .fetchCount(db)
    }
}

extension TransactionsCountQuery: Equatable {}
