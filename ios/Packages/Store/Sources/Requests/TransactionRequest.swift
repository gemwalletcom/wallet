// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct TransactionRequest: DatabaseQueryable {
    private let walletId: WalletId
    private let transactionId: TransactionId

    public var filters: [TransactionsRequestFilter] = []

    public init(
        walletId: WalletId,
        transactionId: TransactionId,
    ) {
        self.walletId = walletId
        self.transactionId = transactionId
    }

    public func fetch(_ db: Database) throws -> TransactionExtended? {
        try TransactionsRequest.fetch(
            db,
            type: .transaction(id: transactionId.identifier),
            filters: filters,
            walletId: walletId,
        ).first
    }
}

extension TransactionRequest: Equatable {}
