// Copyright (c). Gem Wallet. All rights reserved.

import GRDB
import Primitives

public struct WalletsQuery: DatabaseQueryable {
    public init() {}

    public func fetch(_ db: Database) throws -> [Wallet] {
        try WalletRecord
            .including(all: WalletRecord.accounts)
            .asRequest(of: WalletRecordInfo.self)
            .fetchAll(db)
            .map { $0.mapToWallet() }
    }
}

extension WalletsQuery: Equatable {}
