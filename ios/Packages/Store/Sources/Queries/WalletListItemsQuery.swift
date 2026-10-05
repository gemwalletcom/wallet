// Copyright (c). Gem Wallet. All rights reserved.

import GRDB
import Primitives

public struct WalletListItemsQuery: DatabaseQueryable {
    public init() {}

    public func fetch(_ db: Database) throws -> [WalletListItem] {
        try WalletRecord.fetchAll(db).map { $0.toWalletListItem() }
    }
}

extension WalletListItemsQuery: Equatable {}
