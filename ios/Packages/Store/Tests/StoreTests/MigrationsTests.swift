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
            #expect(try! db.tableExists(BannerRecord.databaseTableName))
            #expect(try! db.tableExists(NFTCollectionRecord.databaseTableName))
        }
    }

    @Test
    func runChanges() throws {
        let db = DB.mock()
        var migrations = Migrations()

        try migrations.run(dbQueue: db.dbQueue)
        try migrations.runChanges(dbQueue: db.dbQueue)

        try db.dbQueue.read { db in
            let walletColumns = try db.columns(in: WalletRecord.databaseTableName)
            #expect(walletColumns.contains(where: { $0.name == WalletRecord.Columns.isPinned.name }))

            let balanceColumns = try db.columns(in: BalanceRecord.databaseTableName)
            #expect(balanceColumns.contains(where: { $0.name == BalanceRecord.Columns.availableAmount.name }))
            #expect(balanceColumns.contains(where: { $0.name == BalanceRecord.Columns.isActive.name }))
            #expect(balanceColumns.contains(where: { $0.name == BalanceRecord.Columns.earn.name }))
            #expect(balanceColumns.contains(where: { $0.name == BalanceRecord.Columns.earnAmount.name }))

            let assetColumns = try db.columns(in: AssetRecord.databaseTableName)
            #expect(assetColumns.contains(where: { $0.name == AssetRecord.Columns.isSellable.name }))
            #expect(assetColumns.contains(where: { $0.name == AssetRecord.Columns.isStakeable.name }))
            #expect(assetColumns.contains(where: { $0.name == AssetRecord.Columns.isEarnable.name }))
            #expect(assetColumns.contains(where: { $0.name == AssetRecord.Columns.earnApr.name }))
            #expect(assetColumns.contains(where: { $0.name == AssetRecord.Columns.rank.name }))

            let validatorColumns = try db.columns(in: StakeValidatorRecord.databaseTableName)
            #expect(validatorColumns.contains(where: { $0.name == StakeValidatorRecord.Columns.providerType.name }))

            let priceColumns = try db.columns(in: PriceRecord.databaseTableName)
            #expect(priceColumns.contains(where: { $0.name == PriceRecord.Columns.marketCap.name }))
            #expect(priceColumns.contains(where: { $0.name == PriceRecord.Columns.priceUsd.name }))

            #expect(try! db.tableExists(AssetLinkRecord.databaseTableName))
            #expect(try! db.tableExists(SearchRecord.databaseTableName))
            #expect(try! db.tableExists(FiatRateRecord.databaseTableName))
            #expect(try! db.tableExists(AddressRecord.databaseTableName))
        }
    }

    @Test
    func removeChainRemovesSeiReferencesAndPreservesSeiEvm() throws {
        let store = DB.mock()

        try store.dbQueue.write { db in
            try seedChain(db, chain: "sei", associations: "[]")
            try seedChain(db, chain: "seievm", associations: "[{\"assetId\":\"sei\"}]")
            try db.execute(
                sql: "INSERT INTO contacts (id, name, createdAt, updatedAt) VALUES ('contact', 'Contact', 0, 0)",
            )
            try db.execute(
                sql: "INSERT INTO contacts_addresses (id, contactId, address, chain) VALUES ('sei-contact', 'contact', 'sei-address', 'sei'), ('seievm-contact', 'contact', 'seievm-address', 'seievm')",
            )
        }

        var migrator = DatabaseMigrator()
        migrator.registerMigration("Remove chain", foreignKeyChecks: .immediate) { db in
            try Migrations.removeChain(db, chain: "sei")
        }
        try migrator.migrate(store.dbQueue)

        try store.dbQueue.read { (db: Database) throws in
            #expect(try Int.fetchOne(db, sql: "SELECT COUNT(*) FROM assets WHERE chain = 'sei'") == 0)
            #expect(try Int.fetchOne(db, sql: "SELECT COUNT(*) FROM assets WHERE chain = 'seievm'") == 1)
            #expect(try Int.fetchOne(db, sql: "SELECT COUNT(*) FROM wallets_accounts WHERE chain = 'sei'") == 0)
            #expect(try Int.fetchOne(db, sql: "SELECT COUNT(*) FROM wallets_accounts WHERE chain = 'seievm'") == 1)
            #expect(try Int.fetchOne(db, sql: "SELECT COUNT(*) FROM wallets") == 2)
            #expect(try Int.fetchOne(db, sql: "SELECT COUNT(*) FROM wallets_connections") == 1)
            #expect(try Int.fetchOne(db, sql: "SELECT COUNT(*) FROM notifications") == 1)
            #expect(try Int.fetchOne(db, sql: "SELECT COUNT(*) FROM contacts_addresses") == 1)
            #expect(try String.fetchOne(db, sql: "SELECT associations FROM assets WHERE id = 'seievm'") == "[]")
            #expect(try Row.fetchAll(db, sql: "PRAGMA foreign_key_check").isEmpty)
        }
    }

    private func seedChain(_ db: Database, chain: String, associations: String) throws {
        let walletId = "wallet-\(chain)"
        try db.execute(
            sql: "INSERT INTO wallets (id, name, type, `index`, `order`, isPinned) VALUES (?, ?, 'single', 0, 0, 0)",
            arguments: [walletId, chain],
        )
        try db.execute(
            sql: "INSERT INTO assets (id, chain, name, symbol, decimals, type, isEnabled, isBuyable, isSellable, isSwappable, isStakeable, isEarnable, rank, hasImage, associations) " +
                "VALUES (?, ?, ?, ?, 6, 'native', 1, 0, 0, 0, 0, 0, 1, 0, ?)",
            arguments: [chain, chain, chain, chain, associations],
        )
        try db.execute(
            sql: "INSERT INTO wallets_accounts (walletId, chain, address, `index`, derivationPath) VALUES (?, ?, ?, 0, '')",
            arguments: [walletId, chain, "\(chain)-address"],
        )
        try db.execute(
            sql: "INSERT INTO wallets_connections (id, sessionId, walletId, state, chains, createdAt, expireAt, appName, appDescription, appLink, appIcon) " +
                "VALUES (?, ?, ?, 'active', ?, 0, 1, 'App', 'App', 'https://app.example', 'https://app.example/icon.png')",
            arguments: ["connection-\(chain)", "session-\(chain)", walletId, "[\"\(chain)\"]"],
        )
        try db.execute(
            sql: "INSERT INTO notifications (id, walletId, createdAt, item) VALUES (?, ?, 0, ?)",
            arguments: ["notification-\(chain)", walletId, "{\"icon\":{\"type\":\"asset\",\"value\":\"\(chain)\"}}"],
        )
    }
}
