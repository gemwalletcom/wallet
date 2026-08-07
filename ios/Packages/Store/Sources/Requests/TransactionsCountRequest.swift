// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct TransactionsCountRequest: DatabaseQueryable {
    public var walletId: WalletId
    private let states: [TransactionState]
    private let rank: Int

    public init(
        walletId: WalletId,
        states: [TransactionState],
        rank: Int,
    ) {
        self.walletId = walletId
        self.states = states
        self.rank = rank
    }

    public func fetch(_ db: Database) throws -> Int {
        try TransactionRecord
            .filter(TransactionRecord.Columns.walletId == walletId.id)
            .filter(states.map(\.rawValue).contains(TransactionRecord.Columns.state))
            .joining(required: TransactionRecord.asset.filter(AssetRecord.Columns.rank > rank))
            .fetchCount(db)
    }
}

extension TransactionsCountRequest: Equatable {}
