// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing

struct MigrationsTests {
    @Test
    func run() throws {
        let db = DB.mock()

        try db.dbPool.read { db in
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

        try migrations.run(dbQueue)
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
        try migrations.runChanges(dbQueue)

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
            #expect(assetColumns.contains(where: { $0.name == AssetRecord.Columns.hasCandles.name }))
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
        try migrations.run(dbQueue)
        try migrations.runChanges(dbQueue)

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
        try migrations.run(dbQueue)
        try dbQueue.write { db in
            try db.drop(table: AssetMarketRecord.databaseTableName)
            try db.execute(sql: "DELETE FROM grdb_migrations WHERE identifier = ?", arguments: ["Recreate \(AssetMarketRecord.databaseTableName)"])
        }

        var upgraded = Migrations()
        try upgraded.run(dbQueue)

        #expect(try dbQueue.read { try $0.tableExists(AssetMarketRecord.databaseTableName) })
    }

    @Test
    func transactionsMoveFromTheSingleColumnIndexesToTheWalletDateIndex() throws {
        let table = TransactionRecord.databaseTableName
        let walletId = TransactionRecord.Columns.walletId.name
        let date = TransactionRecord.Columns.date.name
        let dbQueue = try DatabaseQueue()
        var migrations = Migrations()
        try migrations.run(dbQueue)
        try dbQueue.write { db in
            try db.drop(indexOn: table, columns: [walletId, date])
            try db.execute(sql: "CREATE INDEX \(table)_on_\(walletId) ON \(table)(\(walletId))")
            try db.execute(sql: "CREATE INDEX \(table)_on_\(date) ON \(table)(\(date))")
        }

        try migrations.runChanges(dbQueue)

        let indexes = try dbQueue.read { try $0.indexes(on: table).map(\.columns) }
        #expect(indexes.contains([walletId, date]))
        #expect(!indexes.contains([walletId]))
        #expect(!indexes.contains([date]))
    }

    @Test
    func perpetualsAreIndexedByName() throws {
        let table = PerpetualRecord.databaseTableName
        let name = PerpetualRecord.Columns.name.name
        let dbQueue = try DatabaseQueue()
        var migrations = Migrations()
        try migrations.run(dbQueue)
        try migrations.runChanges(dbQueue)
        #expect(try dbQueue.read { try $0.indexes(on: table).map(\.columns) }.contains([name]))

        try dbQueue.write { db in
            try db.drop(indexOn: table, columns: [name])
            try db.execute(sql: "DELETE FROM grdb_migrations WHERE identifier = ?", arguments: ["Add isEnabled and the name index to \(table)"])
        }
        var upgrade = Migrations()
        try upgrade.run(dbQueue)
        try upgrade.runChanges(dbQueue)

        #expect(try dbQueue.read { try $0.indexes(on: table).map(\.columns) }.contains([name]))
    }

    @Test
    func accountsWithoutAnExtendedPublicKeyAreSavedAfterAnUpgradeFromTheNotNullSchema() throws {
        let table = AccountRecord.databaseTableName
        let walletId = "multicoin_0xabc"
        let dbQueue = try DatabaseQueue()
        var migrations = Migrations()
        try migrations.run(dbQueue)
        try migrations.runChanges(dbQueue)
        try dbQueue.write { db in
            try db.drop(table: table)
            try db.execute(sql: """
            CREATE TABLE \(table) (
                walletId TEXT NOT NULL REFERENCES \(WalletRecord.databaseTableName)(id) ON DELETE CASCADE,
                chain TEXT NOT NULL,
                address TEXT NOT NULL,
                extendedPublicKey TEXT NOT NULL,
                publicKey TEXT,
                "index" NUMERIC NOT NULL DEFAULT 0,
                derivationPath TEXT NOT NULL,
                UNIQUE (walletId, chain, derivationPath, address)
            );
            CREATE INDEX \(table)_on_walletId ON \(table)(walletId);
            INSERT INTO \(WalletRecord.databaseTableName) (id, name, type) VALUES ('\(walletId)', 'Main', 'multicoin');
            INSERT INTO \(AssetRecord.databaseTableName) (id, chain, name, symbol, decimals, type) VALUES ('ethereum', 'ethereum', 'Ethereum', 'ETH', 18, 'NATIVE');
            INSERT INTO \(table) (walletId, chain, address, extendedPublicKey, derivationPath) VALUES ('\(walletId)', 'ethereum', '0xabc', 'xpub', '');
            """)
            try db.execute(sql: "DELETE FROM grdb_migrations WHERE identifier = ?", arguments: ["Make extendedPublicKey nullable in \(table)"])
        }

        var upgrade = Migrations()
        try upgrade.run(dbQueue)
        try upgrade.runChanges(dbQueue)
        try dbQueue.write { db in
            try Account.mock(chain: .ethereum, address: "0xdef").toRecord(walletId: walletId).upsert(db)
        }

        let accounts = try dbQueue.read { try AccountRecord.order(AccountRecord.Columns.address).fetchAll($0) }
        #expect(accounts.map(\.address) == ["0xabc", "0xdef"])
        #expect(accounts.map(\.extendedPublicKey) == ["xpub", nil])
    }

    @Test
    func deprecatedChainsLeaveOnlyKeyWalletAccounts() throws {
        let dbQueue = try DatabaseQueue()
        var migrations = Migrations()
        try migrations.run(dbQueue)
        try migrations.runChanges(dbQueue)
        try dbQueue.write { db in
            try db.execute(sql: """
            INSERT INTO \(WalletRecord.databaseTableName) (id, name, type) VALUES
                ('multicoin_0x1', 'Main', 'multicoin'), ('view_blast_0xview', 'View', 'view'), ('privateKey_blast_0xkey', 'Key', 'privateKey');
            INSERT INTO \(AssetRecord.databaseTableName) (id, chain, name, symbol, decimals, type) VALUES
                ('ethereum', 'ethereum', 'Asset', 'AST', 18, 'NATIVE'), ('blast', 'blast', 'Asset', 'AST', 18, 'NATIVE'),
                ('blast_0xtoken', 'blast', 'Asset', 'AST', 18, 'ERC20'), ('sei', 'sei', 'Asset', 'AST', 6, 'NATIVE'),
                ('seievm', 'seievm', 'Asset', 'AST', 18, 'NATIVE'), ('seievm_0xtoken', 'seievm', 'Asset', 'AST', 18, 'ERC20');
            INSERT INTO \(AccountRecord.databaseTableName) (walletId, chain, address, derivationPath) VALUES
                ('multicoin_0x1', 'ethereum', '0x1', ''), ('multicoin_0x1', 'blast', '0x1', ''), ('multicoin_0x1', 'sei', 'sei1', ''),
                ('multicoin_0x1', 'seievm', '0x1', ''), ('view_blast_0xview', 'blast', '0xview', ''), ('privateKey_blast_0xkey', 'blast', '0xkey', '');
            INSERT INTO \(PriceRecord.databaseTableName) (assetId) VALUES ('blast'), ('blast_0xtoken'), ('sei'), ('seievm'), ('seievm_0xtoken');
            INSERT INTO \(BalanceRecord.databaseTableName) (assetId, walletId) VALUES ('blast_0xtoken', 'multicoin_0x1'), ('seievm_0xtoken', 'multicoin_0x1');
            """)
            try db.execute(sql: "DELETE FROM grdb_migrations WHERE identifier = ?", arguments: ["Remove sei and blast"])
        }

        var upgrade = Migrations()
        try upgrade.run(dbQueue)
        try upgrade.runChanges(dbQueue)

        let rows = { (sql: String) in try dbQueue.read { try String.fetchAll($0, sql: sql) } }
        #expect(try rows("SELECT id FROM \(WalletRecord.databaseTableName) ORDER BY id") == ["multicoin_0x1", "privateKey_blast_0xkey"])
        #expect(
            try rows("SELECT walletId || ':' || chain FROM \(AccountRecord.databaseTableName) ORDER BY walletId, chain")
                == ["multicoin_0x1:ethereum", "multicoin_0x1:seievm", "privateKey_blast_0xkey:blast"],
        )
        #expect(try rows("SELECT id FROM \(AssetRecord.databaseTableName) ORDER BY id") == ["blast", "ethereum", "seievm", "seievm_0xtoken"])
        #expect(try rows("SELECT assetId FROM \(PriceRecord.databaseTableName) ORDER BY assetId") == ["seievm", "seievm_0xtoken"])
        #expect(try rows("SELECT assetId FROM \(BalanceRecord.databaseTableName)") == ["seievm_0xtoken"])
        #expect(try rows("SELECT \"table\" FROM pragma_foreign_key_check").isEmpty)
    }
}
