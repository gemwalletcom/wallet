// Copyright (c). Gem Wallet. All rights reserved.

import GRDB
import Primitives

public struct WalletsRequest: DatabaseQueryable {
    private let isPinned: Bool

    public init(isPinned: Bool) {
        self.isPinned = isPinned
    }

    public func fetch(_ db: Database) throws -> [Wallet] {
        try WalletRecord
            .including(all: WalletRecord.accounts)
            .filter(WalletRecord.Columns.isPinned == isPinned)
            .asRequest(of: WalletRecordInfo.self)
            .fetchAll(db)
            .map { $0.mapToWallet() }
    }
}

extension WalletsRequest: Equatable {}
