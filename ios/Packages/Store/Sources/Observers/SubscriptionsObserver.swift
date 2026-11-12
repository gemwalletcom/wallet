// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct SubscriptionsObserver: Sendable {
    private let db: any DatabaseReader

    public init(db: some DatabaseReader) {
        self.db = db
    }

    public func observe() -> AsyncValueObservation<[Account]> {
        ValueObservation
            .tracking {
                try AccountRecord
                    .fetchAll($0)
                    .map { $0.toAccount() }
            }
            .values(in: db)
    }
}
