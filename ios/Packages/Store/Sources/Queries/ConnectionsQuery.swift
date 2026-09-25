// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct ConnectionsQuery: DatabaseQueryable {
    public init() {}

    public func fetch(_ db: Database) throws -> [WalletConnection] {
        try WalletRecord
            .including(required: WalletRecord.connection)
            .including(all: WalletRecord.accounts)
            .asRequest(of: WalletConnectionInfo.self)
            .fetchAll(db)
            .map { $0.mapToWalletConnection() }
    }
}

extension ConnectionsQuery: Equatable {}
