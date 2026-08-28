// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct BannersRequest: DatabaseQueryable {
    public var walletId: WalletId?

    private let assetId: AssetId?
    private let events: [BannerEvent]

    public init(
        walletId: WalletId?,
        assetId: AssetId?,
        events: [BannerEvent],
    ) {
        self.walletId = walletId
        self.assetId = assetId
        self.events = events
    }

    public func fetch(_ db: Database) throws -> [Banner] {
        var query = BannerRecord
            .including(optional: BannerRecord.asset)
            .filter(events.map(\.rawValue).contains(BannerRecord.Columns.event))
            .filter(BannerRecord.Columns.state != BannerState.cancelled.rawValue)
            .asRequest(of: BannerInfo.self)

        if let walletId {
            query = query.filter(BannerRecord.Columns.walletId == walletId.id || BannerRecord.Columns.walletId == nil)
        }
        if let assetId {
            query = query.filter(BannerRecord.Columns.assetId == assetId.identifier)
        }

        return try query
            .fetchAll(db)
            .compactMap { $0.mapToBanner() }
    }
}

extension BannersRequest: Equatable {}
