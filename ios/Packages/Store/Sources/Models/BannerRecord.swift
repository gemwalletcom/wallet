// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct BannerRecord: Codable, FetchableRecord, PersistableRecord {
    public static let databaseTableName: String = "banners"

    public enum Columns {
        static let id = Column("id")
        static let state = Column("state")
        static let event = Column("event")
        static let assetId = Column("assetId")
        static let walletId = Column("walletId")
    }

    public var id: String
    public var walletId: String?
    public var assetId: AssetId?
    public var event: BannerEvent
    public var state: BannerState

    static let asset = belongsTo(AssetRecord.self, key: "asset", using: ForeignKey(["assetId"], to: ["id"]))
}

extension BannerRecord: CreateTable {
    static func create(db: Database) throws {
        try db.create(table: databaseTableName, ifNotExists: true) {
            $0.primaryKey(Columns.id.name, .text)
                .notNull()
                .indexed()
            $0.column(Columns.walletId.name, .text)
                .indexed()
                .references(WalletRecord.databaseTableName, onDelete: .cascade, onUpdate: .cascade)
            $0.column(Columns.assetId.name, .text)
                .references(AssetRecord.databaseTableName, onDelete: .cascade, onUpdate: .cascade)
            $0.column(Columns.event.name, .text)
                .notNull()
            $0.column(Columns.state.name, .text)
                .notNull()
            $0.uniqueKey(
                [
                    Columns.walletId.name,
                    Columns.assetId.name,
                    Columns.event.name,
                ],
            )
        }
    }
}

extension Banner {
    var record: BannerRecord {
        BannerRecord(
            id: id,
            walletId: walletId?.id,
            assetId: asset?.id,
            event: event,
            state: state,
        )
    }
}

extension NewBanner {
    var record: BannerRecord {
        BannerRecord(
            id: [walletId, assetId?.identifier, event.rawValue].compactMap(\.self).joined(separator: "_"),
            walletId: walletId,
            assetId: assetId,
            event: event,
            state: state,
        )
    }
}

public struct NewBanner {
    let walletId: String?
    let assetId: AssetId?
    let event: BannerEvent
    let state: BannerState

    public init(
        walletId: String? = .none,
        assetId: AssetId? = .none,
        event: BannerEvent,
        state: BannerState,
    ) {
        self.walletId = walletId
        self.assetId = assetId
        self.event = event
        self.state = state
    }
}

public extension NewBanner {
    static func stake(assetId: AssetId) -> NewBanner {
        NewBanner(
            assetId: assetId,
            event: .stake,
            state: .active,
        )
    }

    static func accountActivation(assetId: AssetId) -> NewBanner {
        NewBanner(
            assetId: assetId,
            event: .accountActivation,
            state: .active,
        )
    }


    static func onboarding(walletId: WalletId) -> NewBanner {
        NewBanner(
            walletId: walletId.id,
            event: .onboarding,
            state: .active,
        )
    }

    static func tradePerpetuals(assetId: AssetId) -> NewBanner {
        NewBanner(
            assetId: assetId,
            event: .tradePerpetuals,
            state: .active,
        )
    }
}
