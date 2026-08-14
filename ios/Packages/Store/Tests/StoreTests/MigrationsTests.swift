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
    func removeChainsRemovesSeiReferences() throws {
        let store = DB.mock()
        let chain = Migrations.sei

        try store.dbQueue.write { db in
            try seedChain(db, chain: chain)

            try Migrations.removeChains(db, chains: [chain])

            #expect(
                try String.fetchOne(
                    db,
                    sql: """
                    SELECT 'assets=' || (SELECT COUNT(*) FROM assets WHERE chain = '\(chain)') ||
                        ',accounts=' || (SELECT COUNT(*) FROM wallets_accounts WHERE chain = '\(chain)') ||
                        ',connections=' || (SELECT COUNT(*) FROM wallets_connections WHERE instr(chains, '"\(chain)"') > 0) ||
                        ',notifications=' || (SELECT COUNT(*) FROM notifications WHERE instr(item, '"\(chain)"') > 0)
                    """,
                ) == "assets=0,accounts=0,connections=0,notifications=0",
            )
        }
    }

    private func seedChain(_ db: Database, chain: String) throws {
        let walletId = "wallet-\(chain)"
        try db.execute(
            sql: "INSERT INTO wallets (id, name, type, `index`, `order`, isPinned) VALUES (?, ?, 'single', 0, 0, 0)",
            arguments: [walletId, chain],
        )
        try db.execute(
            sql: "INSERT INTO assets (id, chain, name, symbol, decimals, type, isEnabled, isBuyable, isSellable, isSwappable, isStakeable, isEarnable, rank, hasImage, associations) " +
                "VALUES (?, ?, ?, ?, 6, 'native', 1, 0, 0, 0, 0, 0, 1, 0, ?)",
            arguments: [chain, chain, chain, chain, "[]"],
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
