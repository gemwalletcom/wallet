// Copyright (c). Gem Wallet. All rights reserved.

import GRDB
import Primitives

public struct WalletsRequest: DatabaseQueryable {
    private let isPinned: Bool?

    public init(isPinned: Bool?) {
        self.isPinned = isPinned
    }

    public func fetch(_ db: Database) throws -> [Wallet] {
        var request = WalletRecord
            .including(all: WalletRecord.accounts)
        if let isPinned {
            request = request.filter(WalletRecord.Columns.isPinned == isPinned)
        }
        return try request
            .asRequest(of: WalletRecordInfo.self)
            .fetchAll(db)
            .map { $0.mapToWallet() }
    }
}

extension WalletsRequest: Equatable {}
