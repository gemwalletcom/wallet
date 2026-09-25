// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct PerpetualsQuery: DatabaseQueryable {
    public let searchQuery: String
    public let limit: Int
    public let requiresVolume: Bool

    public init(searchQuery: String, limit: Int, requiresVolume: Bool) {
        self.searchQuery = searchQuery
        self.limit = limit
        self.requiresVolume = requiresVolume
    }

    public func fetch(_ db: Database) throws -> [PerpetualData] {
        var request = PerpetualRecord.including(required: PerpetualRecord.asset)

        if requiresVolume {
            request = request.filter(PerpetualRecord.Columns.volume24h > 0 || PerpetualRecord.Columns.isPinned)
        }

        if !searchQuery.isEmpty {
            request = request.filter(
                PerpetualRecord.Columns.name.like("%%\(searchQuery)%%") ||
                    TableAlias(name: AssetRecord.databaseTableName)[AssetRecord.Columns.symbol].like("%%\(searchQuery)%%"),
            )
        }

        return try request
            .order(PerpetualRecord.Columns.isPinned.desc, PerpetualRecord.Columns.volume24h.desc)
            .limit(limit)
            .asRequest(of: PerpetualInfo.self)
            .fetchAll(db)
            .map { $0.mapToPerpetualData() }
    }
}

extension PerpetualsQuery: Equatable {}
