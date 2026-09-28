// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

struct AssetMarketRecord: Codable, FetchableRecord, PersistableRecord {
    static let databaseTableName: String = "asset_market"

    enum Columns {
        static let assetId = Column("assetId")
        static let marketCap = Column("marketCap")
        static let marketCapFdv = Column("marketCapFdv")
        static let marketCapRank = Column("marketCapRank")
        static let totalVolume = Column("totalVolume")
        static let circulatingSupply = Column("circulatingSupply")
        static let totalSupply = Column("totalSupply")
        static let maxSupply = Column("maxSupply")
        static let allTimeHigh = Column("allTimeHigh")
        static let allTimeHighDate = Column("allTimeHighDate")
        static let allTimeHighChangePercentage = Column("allTimeHighChangePercentage")
        static let allTimeLow = Column("allTimeLow")
        static let allTimeLowDate = Column("allTimeLowDate")
        static let allTimeLowChangePercentage = Column("allTimeLowChangePercentage")
    }

    var assetId: AssetId

    var marketCap: Double?
    var marketCapFdv: Double?
    var marketCapRank: Int?
    var totalVolume: Double?
    var circulatingSupply: Double?
    var totalSupply: Double?
    var maxSupply: Double?
    var allTimeHigh: Double?
    var allTimeHighDate: Date?
    var allTimeHighChangePercentage: Double?
    var allTimeLow: Double?
    var allTimeLowDate: Date?
    var allTimeLowChangePercentage: Double?
}

extension AssetMarketRecord: CreateTable {
    static func create(db: Database) throws {
        try db.create(table: databaseTableName, ifNotExists: true) {
            $0.column(Columns.assetId.name, .text)
                .primaryKey()
                .references(AssetRecord.databaseTableName, onDelete: .cascade, onUpdate: .cascade)

            $0.column(Columns.marketCap.name, .double)
            $0.column(Columns.marketCapFdv.name, .double)
            $0.column(Columns.marketCapRank.name, .integer)
            $0.column(Columns.totalVolume.name, .double)
            $0.column(Columns.circulatingSupply.name, .double)
            $0.column(Columns.totalSupply.name, .double)
            $0.column(Columns.maxSupply.name, .double)
            $0.column(Columns.allTimeHigh.name, .double)
            $0.column(Columns.allTimeHighDate.name, .date)
            $0.column(Columns.allTimeHighChangePercentage.name, .double)
            $0.column(Columns.allTimeLow.name, .double)
            $0.column(Columns.allTimeLowDate.name, .date)
            $0.column(Columns.allTimeLowChangePercentage.name, .double)
        }
    }
}

extension AssetMarketRecord: Identifiable {
    var id: String {
        assetId.identifier
    }
}

extension AssetMarketRecord {
    init(assetId: AssetId, market: AssetMarket) {
        self.init(
            assetId: assetId,
            marketCap: market.marketCap,
            marketCapFdv: market.marketCapFdv,
            marketCapRank: market.marketCapRank.map { Int($0) },
            totalVolume: market.totalVolume,
            circulatingSupply: market.circulatingSupply,
            totalSupply: market.totalSupply,
            maxSupply: market.maxSupply,
            allTimeHigh: market.allTimeHighValue.map { Double($0.value) },
            allTimeHighDate: market.allTimeHighValue?.date,
            allTimeHighChangePercentage: market.allTimeHighValue.map { Double($0.percentage) },
            allTimeLow: market.allTimeLowValue.map { Double($0.value) },
            allTimeLowDate: market.allTimeLowValue?.date,
            allTimeLowChangePercentage: market.allTimeLowValue.map { Double($0.percentage) },
        )
    }

    func mapToMarket() -> AssetMarket {
        AssetMarket(
            marketCap: marketCap,
            marketCapFdv: marketCapFdv,
            marketCapRank: marketCapRank?.asInt32,
            totalVolume: totalVolume,
            circulatingSupply: circulatingSupply,
            totalSupply: totalSupply,
            maxSupply: maxSupply,
            allTimeHighValue: allTimeHigh.flatMap { value in
                allTimeHighDate.map { ChartValuePercentage(date: $0, value: Float(value), percentage: Float(allTimeHighChangePercentage ?? 0)) }
            },
            allTimeLowValue: allTimeLow.flatMap { value in
                allTimeLowDate.map { ChartValuePercentage(date: $0, value: Float(value), percentage: Float(allTimeLowChangePercentage ?? 0)) }
            },
        )
    }
}
