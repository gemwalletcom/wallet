// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct TransactionsCountRequest: DatabaseQueryable {
    public var walletId: WalletId
    private let type: TransactionsRequestType
    private let filters: [TransactionsRequestFilter]

    public init(
        walletId: WalletId,
        type: TransactionsRequestType,
        filters: [TransactionsRequestFilter] = [],
    ) {
        self.walletId = walletId
        self.type = type
        self.filters = filters
    }

    public func fetch(_ db: Database) throws -> Int {
        try TransactionsRequest.query(walletId: walletId, type: type, filters: filters)
            .fetchCount(db)
    }
}

extension TransactionsCountRequest: Equatable {}
