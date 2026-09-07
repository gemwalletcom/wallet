// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct TransactionRequest: DatabaseQueryable {
    private let walletId: WalletId
    private let recordId: UInt64

    public init(walletId: WalletId, recordId: UInt64) {
        self.walletId = walletId
        self.recordId = recordId
    }

    public func fetch(_ db: Database) throws -> TransactionExtended {
        let request = TransactionRecord
            .filter(TransactionRecord.Columns.walletId == walletId.id)
            .filter(TransactionRecord.Columns.id == recordId)
        guard let transaction = try TransactionsRequest.fetch(db, request: request).first else {
            throw RecordError.recordNotFound(databaseTableName: TransactionRecord.databaseTableName, key: [:])
        }
        return transaction
    }
}

extension TransactionRequest: Equatable {}
