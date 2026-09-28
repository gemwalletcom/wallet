// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
@testable import Store
import StoreTestKit
import Testing

struct MigrationsTests {
    @Test
    func run() throws {
        let db = DB.mock()

        try db.dbQueue.read { db in
            #expect(try! db.tableExists(WalletRecord.databaseTableName))
            #expect(try! db.tableExists(AccountRecord.databaseTableName))
            #expect(try! db.tableExists(AssetRecord.databaseTableName))
            #expect(try! db.tableExists(BalanceRecord.databaseTableName))
            #expect(try! db.tableExists(TransactionRecord.databaseTableName))
            #expect(try! db.tableExists(NodeRecord.databaseTableName))
            #expect(!(try! db.tableExists("nodes_selected")))
            #expect(try! db.tableExists(BannerRecord.databaseTableName))
            #expect(try! db.tableExists(NFTCollectionRecord.databaseTableName))
        }
    }

    @Test
    func runChanges() throws {
        let dbQueue = try DatabaseQueue()
        var migrations = Migrations()

        try migrations.run(dbQueue: dbQueue)
        try dbQueue.write { db in
            try db.create(table: "nodes_selected") {
                $0.column("chain", .text)
            }
            try db.create(table: "nodes_selected_v1") {
                $0.column("chain", .text)
            }
            try db.execute(sql: "INSERT INTO assets (id, chain, name, symbol, decimals, type) VALUES ('ethereum', 'ethereum', 'Ethereum', 'ETH', 18, 'NATIVE')")
            try db.execute(sql: "INSERT INTO price_alerts (id, assetId, currency, price, priceDirection) VALUES ('ethereum_USD_1e-05_up', 'ethereum', 'USD', 0.00001, 'up')")

            try db.alter(table: PriceRecord.databaseTableName) {
                $0.add(column: AssetMarketRecord.Columns.marketCap.name, .double)
                $0.add(column: AssetMarketRecord.Columns.marketCapRank.name, .integer)
                $0.add(column: AssetMarketRecord.Columns.allTimeHigh.name, .double)
                $0.add(column: AssetMarketRecord.Columns.allTimeHighDate.name, .date)
            }
            try db.execute(sql: "INSERT INTO prices (assetId, price, priceUsd, priceChangePercentage24h, marketCap, marketCapRank, allTimeHigh, allTimeHighDate) VALUES ('ethereum', 2, 2, 1, 42, 7, 4800, 0)")
        }
        try migrations.runChanges(dbQueue: dbQueue)

        try dbQueue.read { db in
            let walletColumns = try db.columns(in: WalletRecord.databaseTableName)
            #expect(walletColumns.contains(where: { $0.name == WalletRecord.Columns.isPinned.name }))

            let balanceColumns = try db.columns(in: BalanceRecord.databaseTableName)
            #expect(balanceColumns.contains(where: { $0.name == BalanceRecord.Columns.availableAmount.name }))
            #expect(balanceColumns.contains(where: { $0.name == BalanceRecord.Columns.isActive.name }))
            #expect(balanceColumns.contains(where: { $0.name == BalanceRecord.Columns.earn.name }))
            #expect(balanceColumns.contains(where: { $0.name == BalanceRecord.Columns.earnAmount.name }))
            #expect(!balanceColumns.contains(where: { $0.name == "lastUsedAt" }), "nothing reads the column, so it is gone")

            let assetColumns = try db.columns(in: AssetRecord.databaseTableName)
            #expect(assetColumns.contains(where: { $0.name == AssetRecord.Columns.isSellable.name }))
            #expect(assetColumns.contains(where: { $0.name == AssetRecord.Columns.isStakeable.name }))
            #expect(assetColumns.contains(where: { $0.name == AssetRecord.Columns.isEarnable.name }))
            #expect(assetColumns.contains(where: { $0.name == AssetRecord.Columns.earnApr.name }))
            #expect(assetColumns.contains(where: { $0.name == AssetRecord.Columns.rank.name }))

            let validatorColumns = try db.columns(in: StakeValidatorRecord.databaseTableName)
            #expect(validatorColumns.contains(where: { $0.name == StakeValidatorRecord.Columns.providerType.name }))

            let priceColumns = try db.columns(in: PriceRecord.databaseTableName).map(\.name)
            #expect(priceColumns.contains(PriceRecord.Columns.priceUsd.name))
            #expect(!priceColumns.contains(AssetMarketRecord.Columns.marketCap.name), "market data lives in its own table")
            #expect(try db.tableExists(AssetMarketRecord.databaseTableName))

            #expect(try! db.tableExists(AssetLinkRecord.databaseTableName))
            #expect(try! db.tableExists(SearchRecord.databaseTableName))
            #expect(try! db.tableExists(FiatRateRecord.databaseTableName))
            #expect(try! db.tableExists(AddressRecord.databaseTableName))
            #expect(!(try! db.tableExists("nodes_selected")))
            #expect(!(try! db.tableExists("nodes_selected_v1")))
            #expect(try PriceAlertRecord.fetchCount(db) == 0, "alerts stored under the app's old identifier format are dropped and come back from the next sync")
        }
    }

    @Test
    func upgradeFromAnOldSchema() throws {
        let dbQueue = try DatabaseQueue()
        try dbQueue.write { db in
            try db.execute(sql: """
            CREATE TABLE grdb_migrations (identifier TEXT NOT NULL PRIMARY KEY);
            INSERT INTO grdb_migrations (identifier) VALUES ('Create all start table');
            CREATE TABLE \(WalletRecord.databaseTableName) (id TEXT PRIMARY KEY, name TEXT, type TEXT, "index" INTEGER, "order" INTEGER);
            CREATE TABLE \(AccountRecord.databaseTableName) (walletId TEXT, chain TEXT, address TEXT);
            CREATE TABLE \(AssetRecord.databaseTableName) (id TEXT PRIMARY KEY, chain TEXT, name TEXT, symbol TEXT, decimals INTEGER, type TEXT);
            CREATE TABLE \(BalanceRecord.databaseTableName) (assetId TEXT, walletId TEXT, available TEXT, frozen TEXT, locked TEXT, staked TEXT, pending TEXT);
            CREATE TABLE \(PriceRecord.databaseTableName) (assetId TEXT PRIMARY KEY, price DOUBLE, priceChangePercentage24h DOUBLE);
            CREATE TABLE \(TransactionRecord.databaseTableName) (id TEXT PRIMARY KEY, walletId TEXT, assetId TEXT, chain TEXT);
            CREATE TABLE \(StakeValidatorRecord.databaseTableName) (id TEXT PRIMARY KEY, chain TEXT);
            INSERT INTO \(AssetRecord.databaseTableName) (id, chain, name, symbol, decimals, type) VALUES ('ethereum', 'ethereum', 'Ethereum', 'ETH', 18, 'NATIVE');
            INSERT INTO \(PriceRecord.databaseTableName) (assetId, price, priceChangePercentage24h) VALUES ('ethereum', 2, 1);
            """)
        }

        var migrations = Migrations()
        try migrations.run(dbQueue: dbQueue)
        try migrations.runChanges(dbQueue: dbQueue)

        try dbQueue.read { db in
            let balanceColumns = try db.columns(in: BalanceRecord.databaseTableName).map(\.name)
            #expect(balanceColumns.contains(BalanceRecord.Columns.earnAmount.name))
            #expect(balanceColumns.contains(BalanceRecord.Columns.totalAmount.name))
            #expect(!balanceColumns.contains("lastUsedAt"))
            #expect(try db.columns(in: WalletRecord.databaseTableName).map(\.name).contains(WalletRecord.Columns.isPinned.name))
            #expect(try db.columns(in: StakeValidatorRecord.databaseTableName).map(\.name).contains(StakeValidatorRecord.Columns.providerType.name))
            #expect(try !(db.columns(in: PriceRecord.databaseTableName).map(\.name).contains(AssetMarketRecord.Columns.marketCap.name)))
            #expect(try db.tableExists(AssetMarketRecord.databaseTableName))
            #expect(try db.tableExists(AddressRecord.databaseTableName))
            #expect(try PriceRecord.fetchCount(db) == 1)
        }
    }

    @Test
    func recreateMarketTableOnAnUpgradeWithoutIt() throws {
        let dbQueue = try DatabaseQueue()
        var migrations = Migrations()
        try migrations.run(dbQueue: dbQueue)
        try dbQueue.write { db in
            try db.drop(table: AssetMarketRecord.databaseTableName)
            try db.execute(sql: "DELETE FROM grdb_migrations WHERE identifier = ?", arguments: ["Recreate \(AssetMarketRecord.databaseTableName)"])
        }

        var upgraded = Migrations()
        try upgraded.run(dbQueue: dbQueue)

        #expect(try dbQueue.read { try $0.tableExists(AssetMarketRecord.databaseTableName) })
    }

    @Test
    func transactionsMoveFromTheSingleColumnIndexesToTheWalletDateIndex() throws {
        let table = TransactionRecord.databaseTableName
        let walletId = TransactionRecord.Columns.walletId.name
        let date = TransactionRecord.Columns.date.name
        let dbQueue = try DatabaseQueue()
        var migrations = Migrations()
        try migrations.run(dbQueue: dbQueue)
        try dbQueue.write { db in
            try db.drop(indexOn: table, columns: [walletId, date])
            try db.execute(sql: "CREATE INDEX \(table)_on_\(walletId) ON \(table)(\(walletId))")
            try db.execute(sql: "CREATE INDEX \(table)_on_\(date) ON \(table)(\(date))")
        }

        try migrations.runChanges(dbQueue: dbQueue)

        let indexes = try dbQueue.read { try $0.indexes(on: table).map(\.columns) }
        #expect(indexes.contains([walletId, date]))
        #expect(!indexes.contains([walletId]))
        #expect(!indexes.contains([date]))
    }
}
