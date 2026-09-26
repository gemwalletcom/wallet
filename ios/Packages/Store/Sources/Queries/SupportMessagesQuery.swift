// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct SupportMessagesQuery: DatabaseQueryable {
    public init() {}

    public func fetch(_ db: Database) throws -> [SupportMessage] {
        try SupportMessageRecord
            .order(SupportMessageRecord.Columns.createdAt.asc)
            .fetchAll(db)
            .map(\.message)
    }
}

extension SupportMessagesQuery: Equatable {}
