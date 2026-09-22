// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

struct PriceRecord: Codable, FetchableRecord, PersistableRecord {
    static let databaseTableName: String = "prices"

    enum Columns {
        static let assetId = Column("assetId")
        static let price = Column("price")
        static let priceUsd = Column("priceUsd")
        static let priceChangePercentage24h = Column("priceChangePercentage24h")
        static let updatedAt = Column("updatedAt")
    }

    var assetId: AssetId
    var price: Double
    var priceUsd: Double
    var priceChangePercentage24h: Double

    var updatedAt: Date?
}

extension PriceRecord: CreateTable {
    static func create(db: Database) throws {
        try db.create(table: databaseTableName, ifNotExists: true) {
            $0.column(Columns.assetId.name, .text)
                .primaryKey()
                .references(AssetRecord.databaseTableName, onDelete: .cascade, onUpdate: .cascade)
            $0.column(Columns.price.name, .numeric)
                .notNull()
                .defaults(to: 0)
            $0.column(Columns.priceUsd.name, .numeric)
                .notNull()
                .defaults(to: 0)
            $0.column(Columns.priceChangePercentage24h.name, .numeric)
                .notNull()
                .defaults(to: 0)

            $0.column(Columns.updatedAt.name, .date)
        }
    }
}

extension PriceRecord: Identifiable {
    var id: String {
        assetId.identifier
    }
}

extension PriceUpdate {
    var record: PriceRecord {
        PriceRecord(
            assetId: assetId,
            price: price,
            priceUsd: priceUsd,
            priceChangePercentage24h: priceChangePercentage24h,
            updatedAt: updatedAt,
        )
    }
}

extension PriceRecord {
    func mapToPrice() -> Price? {
        guard price > 0 else { return nil }
        return Price(
            price: price,
            priceChangePercentage24h: priceChangePercentage24h,
            updatedAt: updatedAt ?? .now,
        )
    }

    func mapToAssetPrice() -> AssetPrice {
        AssetPrice(
            assetId: assetId,
            price: price,
            priceChangePercentage24h: priceChangePercentage24h,
            updatedAt: updatedAt ?? Date(timeIntervalSince1970: 0),
        )
    }
}
