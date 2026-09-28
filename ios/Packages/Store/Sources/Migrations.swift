// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

struct Migrations {
    var migrator = DatabaseMigrator()

    init(
        migrator: DatabaseMigrator = DatabaseMigrator(),
    ) {
        self.migrator = migrator
    }

    private static func clearChainData(_ db: Database, chain: String) throws {
        let byAssetId = [
            TransactionAssetAssociationRecord.databaseTableName,
            BalanceRecord.databaseTableName,
            PriceRecord.databaseTableName,
            AssetLinkRecord.databaseTableName,
            PerpetualRecord.databaseTableName,
        ]
        let byChain = [
            TransactionRecord.databaseTableName,
            AssetRecord.databaseTableName,
        ]
        for tableName in byAssetId where try db.tableExists(tableName) {
            try db.execute(sql: "DELETE FROM \(tableName) WHERE assetId LIKE ? COLLATE NOCASE", arguments: ["\(chain)%"])
        }
        for tableName in byChain where try db.tableExists(tableName) {
            try db.execute(sql: "DELETE FROM \(tableName) WHERE chain = ?", arguments: [chain])
        }
    }

    private static func clearTables(_ db: Database, tableNames: [String]) throws {
        for tableName in tableNames where try db.tableExists(tableName) {
            try db.execute(sql: "DELETE FROM \(tableName)")
        }
    }

    private static func replaceTotalAmount(_ db: Database) throws {
        let table = BalanceRecord.databaseTableName
        guard try db.hasColumn(BalanceRecord.Columns.earnAmount.name, in: table) else { return }
        if try db.hasColumn(BalanceRecord.Columns.totalAmount.name, in: table) {
            try db.alter(table: table) { $0.drop(column: BalanceRecord.Columns.totalAmount.name) }
        }
        try db.alter(table: table) { $0.addColumn(sql: BalanceRecord.totalAmountSQlCreation) }
    }

    mutating func run(dbQueue: DatabaseQueue) throws {
        migrator.registerMigration("Create all start table") { db in
            // wallet
            try WalletRecord.create(db: db)
            try AssetRecord.create(db: db)
            try AccountRecord.create(db: db)
            try BalanceRecord.create(db: db)

            // asset
            try FiatRateRecord.create(db: db)
            try PriceRecord.create(db: db)
            try AssetMarketRecord.create(db: db)
            try AssetLinkRecord.create(db: db)

            // transactions
            try TransactionRecord.create(db: db)
            try TransactionAssetAssociationRecord.create(db: db)
            try AddressRecord.create(db: db)

            // nodes
            try NodeRecord.create(db: db)

            // stake
            try StakeValidatorRecord.create(db: db)
            try StakeDelegationRecord.create(db: db)

            // connections
            try WalletConnectionRecord.create(db: db)

            // others
            try BannerRecord.create(db: db)
            try PriceAlertRecord.create(db: db)
            try ContactRecord.create(db: db)
            try ContactAddressRecord.create(db: db)

            // nft
            try NFTCollectionRecord.create(db: db)
            try NFTAssetRecord.create(db: db)
            try NFTAssetAssociationRecord.create(db: db)

            // perpetuals
            try PerpetualRecord.create(db: db)
            try PerpetualPositionRecord.create(db: db)

            try RecentActivityRecord.create(db: db)
            try AssetListRecord.create(db: db)
            try SearchRecord.create(db: db)
            try NotificationRecord.create(db: db)
            try FiatTransactionRecord.create(db: db)
            try SupportMessageRecord.create(db: db)
        }
        migrator.registerMigration("Recreate \(AssetMarketRecord.databaseTableName)") { db in
            try db.dropTableIfExists(AssetMarketRecord.databaseTableName)
            try AssetMarketRecord.create(db: db)
        }

        try migrator.migrate(dbQueue)
    }

    mutating func runChanges(dbQueue: DatabaseQueue) throws {
        migrator.registerMigration("Delete missing assetId in \(PriceRecord.databaseTableName), \(BalanceRecord.databaseTableName)") {
            try $0.execute(sql: "DELETE FROM prices WHERE assetId NOT IN (SELECT id FROM assets)")
            try $0.execute(sql: "DELETE FROM balances WHERE assetId NOT IN (SELECT id FROM assets)")
        }

        migrator.registerMigration("Add isPinned to \(WalletRecord.databaseTableName)") { db in
            try db.addColumnIfMissing(WalletRecord.Columns.isPinned.name, .boolean, to: WalletRecord.databaseTableName) { $0.defaults(to: false) }
        }

        migrator.registerMigration("Set order as index in \(WalletRecord.databaseTableName)") { db in
            guard try db.hasColumn(WalletRecord.Columns.index.name, in: WalletRecord.databaseTableName),
                  try db.hasColumn(WalletRecord.Columns.order.name, in: WalletRecord.databaseTableName) else { return }
            try db.execute(sql: "UPDATE wallets SET \"order\" = \"index\"")
        }

        migrator.registerMigration("Create \(PriceAlertRecord.databaseTableName)") { db in
            try db.createIfMissing(PriceAlertRecord.self)
        }

        migrator.registerMigration("Recreate \(BannerRecord.databaseTableName)") { db in
            try db.dropTableIfExists(BannerRecord.databaseTableName)
            try BannerRecord.create(db: db)
        }

        migrator.registerMigration("Add balances value to \(BalanceRecord.databaseTableName)") { db in
            let amounts = [
                BalanceRecord.Columns.availableAmount,
                BalanceRecord.Columns.frozenAmount,
                BalanceRecord.Columns.lockedAmount,
                BalanceRecord.Columns.stakedAmount,
                BalanceRecord.Columns.pendingAmount,
                BalanceRecord.Columns.rewardsAmount,
                BalanceRecord.Columns.reservedAmount,
            ]
            for amount in amounts {
                try db.addColumnIfMissing(amount.name, .double, to: BalanceRecord.databaseTableName) { $0.defaults(to: 0) }
            }
            try Self.replaceTotalAmount(db)
        }

        migrator.registerMigration("Add rewards to \(BalanceRecord.databaseTableName)") { db in
            try db.addColumnIfMissing(BalanceRecord.Columns.rewards.name, .text, to: BalanceRecord.databaseTableName) { $0.defaults(to: "0") }
        }

        migrator.registerMigration("Add reserved to \(BalanceRecord.databaseTableName)") { db in
            try db.addColumnIfMissing(BalanceRecord.Columns.reserved.name, .text, to: BalanceRecord.databaseTableName) { $0.defaults(to: "0") }
        }

        migrator.registerMigration("Add updatedAt to \(BalanceRecord.databaseTableName)") { db in
            try db.addColumnIfMissing(BalanceRecord.Columns.updatedAt.name, .date, to: BalanceRecord.databaseTableName)
        }

        migrator.registerMigration("Add isSellable to \(AssetRecord.databaseTableName)") { db in
            try db.addColumnIfMissing(AssetRecord.Columns.isSellable.name, .boolean, to: AssetRecord.databaseTableName) { $0.defaults(to: false) }
        }

        migrator.registerMigration("Add isStakeable to \(AssetRecord.databaseTableName)") { db in
            try db.addColumnIfMissing(AssetRecord.Columns.isStakeable.name, .boolean, to: AssetRecord.databaseTableName) { $0.defaults(to: false) }
        }

        migrator.registerMigration("Add rank to \(AssetRecord.databaseTableName)") { db in
            try db.addColumnIfMissing(AssetRecord.Columns.rank.name, .numeric, to: AssetRecord.databaseTableName) { $0.defaults(to: 0) }
        }

        migrator.registerMigration("Add lastUsedAt to \(BalanceRecord.databaseTableName)") { db in
            try db.addColumnIfMissing("lastUsedAt", .date, to: BalanceRecord.databaseTableName)
        }

        migrator.registerMigration("Create \(AssetLinkRecord.databaseTableName)") { db in
            try db.createIfMissing(AssetLinkRecord.self)
        }

        migrator.registerMigration("Add market values to prices table \(PriceRecord.databaseTableName)") { db in
            try db.addColumnIfMissing(AssetMarketRecord.Columns.marketCap.name, .double, to: PriceRecord.databaseTableName)
            try db.addColumnIfMissing(AssetMarketRecord.Columns.marketCapRank.name, .integer, to: PriceRecord.databaseTableName)
            try db.addColumnIfMissing(AssetMarketRecord.Columns.totalVolume.name, .double, to: PriceRecord.databaseTableName)
            try db.addColumnIfMissing(AssetMarketRecord.Columns.circulatingSupply.name, .double, to: PriceRecord.databaseTableName)
            try db.addColumnIfMissing(AssetMarketRecord.Columns.totalSupply.name, .double, to: PriceRecord.databaseTableName)
            try db.addColumnIfMissing(AssetMarketRecord.Columns.maxSupply.name, .double, to: PriceRecord.databaseTableName)
        }

        migrator.registerMigration("Add stakingApr to \(AssetRecord.databaseTableName)") { db in
            try db.addColumnIfMissing(AssetRecord.Columns.stakingApr.name, .double, to: AssetRecord.databaseTableName)
        }

        migrator.registerMigration("Update \(BalanceRecord.Columns.totalAmount.name) column") { db in
            try Self.replaceTotalAmount(db)
        }

        migrator.registerMigration("Add isActive to \(BalanceRecord.databaseTableName)") { db in
            try db.addColumnIfMissing(BalanceRecord.Columns.isActive.name, .boolean, to: BalanceRecord.databaseTableName) { $0.defaults(to: true) }
        }

        migrator.registerMigration("Add marketCapFdv table \(PriceRecord.databaseTableName)") { db in
            try db.addColumnIfMissing(AssetMarketRecord.Columns.marketCapFdv.name, .double, to: PriceRecord.databaseTableName)
        }

        // not relevant for new users, only debug
        migrator.registerMigration("Add initial nft setup tables drop") { db in
            try db.dropTableIfExists(NFTCollectionRecord.databaseTableName)
            try db.dropTableIfExists(NFTAssetRecord.databaseTableName)
            try db.dropTableIfExists(NFTAssetAssociationRecord.databaseTableName)
            try db.dropTableIfExists("nft_collection_images")
            try db.dropTableIfExists("nft_images")
            try db.dropTableIfExists("nft_attributes")
        }

        migrator.registerMigration("Add initial nft tables setup") { db in
            try NFTCollectionRecord.create(db: db)
            try NFTAssetRecord.create(db: db)
            try NFTAssetAssociationRecord.create(db: db)
        }

        migrator.registerMigration("Add links to \(NFTCollectionRecord.databaseTableName)") { db in
            try db.addColumnIfMissing(NFTCollectionRecord.Columns.links.name, .jsonText, to: NFTCollectionRecord.databaseTableName)
        }

        migrator.registerMigration("Add attributes to \(NFTAssetRecord.databaseTableName)") { db in
            try db.dropTableIfExists("nft_attributes")
            try db.addColumnIfMissing(NFTAssetRecord.Columns.attributes.name, .jsonText, to: NFTAssetRecord.databaseTableName)
        }

        migrator.registerMigration("Add contractAddress to \(NFTAssetRecord.databaseTableName)") { db in
            try db.addColumnIfMissing(NFTAssetRecord.Columns.contractAddress.name, .text, to: NFTAssetRecord.databaseTableName)
        }

        migrator.registerMigration("Add imageUrl to \(WalletRecord.databaseTableName)") { db in
            try db.addColumnIfMissing(WalletRecord.Columns.imageUrl.name, .text, to: WalletRecord.databaseTableName)
            try db.addColumnIfMissing(WalletRecord.Columns.updatedAt.name, .date, to: WalletRecord.databaseTableName)
        }

        migrator.registerMigration("Add currency to \(PriceAlertRecord.databaseTableName)") { db in
            try db.addColumnIfMissing(PriceAlertRecord.Columns.currency.name, .text, to: PriceAlertRecord.databaseTableName) { $0.defaults(to: "USD") }
        }

        migrator.registerMigration("Re-create nft tables") { db in
            try db.dropTableIfExists(NFTAssetAssociationRecord.databaseTableName)
            try db.dropTableIfExists(NFTAssetRecord.databaseTableName)
            try db.dropTableIfExists(NFTCollectionRecord.databaseTableName)

            try NFTCollectionRecord.create(db: db)
            try NFTAssetRecord.create(db: db)
            try NFTAssetAssociationRecord.create(db: db)
        }

        migrator.registerMigration("Add fiat rates") { db in
            try db.createIfMissing(FiatRateRecord.self)
        }

        migrator.registerMigration("Add priceUsd to prices table \(PriceRecord.databaseTableName)") { db in
            try db.addColumnIfMissing(PriceRecord.Columns.priceUsd.name, .double, to: PriceRecord.databaseTableName) { $0.notNull().defaults(to: 0) }
        }

        migrator.registerMigration("Add updatedAt to \(PriceRecord.databaseTableName)") { db in
            try db.addColumnIfMissing(PriceRecord.Columns.updatedAt.name, .date, to: PriceRecord.databaseTableName)
        }

        migrator.registerMigration("Add \(AddressRecord.databaseTableName) table") { db in
            try db.createIfMissing(AddressRecord.self)
        }

        migrator.registerMigration("Add Perpetuals tables") { db in
            try Self.clearChainData(db, chain: "hypercore")

            try db.dropTableIfExists(PerpetualRecord.databaseTableName)
            try db.dropTableIfExists(PerpetualPositionRecord.databaseTableName)

            try PerpetualRecord.create(db: db)
            try PerpetualPositionRecord.create(db: db)
        }

        migrator.registerMigration("Add withdrawable to \(BalanceRecord.databaseTableName)") { db in
            try db.addColumnIfMissing(BalanceRecord.Columns.withdrawable.name, .text, to: BalanceRecord.databaseTableName) { $0.defaults(to: "0") }
            try db.addColumnIfMissing(BalanceRecord.Columns.withdrawableAmount.name, .double, to: BalanceRecord.databaseTableName) { $0.defaults(to: 0) }
        }

        migrator.registerMigration("Add metadata to \(BalanceRecord.databaseTableName)") { db in
            try db.addColumnIfMissing(BalanceRecord.Columns.metadata.name, .jsonText, to: BalanceRecord.databaseTableName)
        }

        migrator.registerMigration("Clear metadata from \(BalanceRecord.databaseTableName)") { db in
            guard try db.hasColumn(BalanceRecord.Columns.metadata.name, in: BalanceRecord.databaseTableName) else { return }
            try db.execute(sql: "UPDATE \(BalanceRecord.databaseTableName) SET metadata = NULL WHERE metadata IS NOT NULL")
        }

        migrator.registerMigration("Add isEnabled to \(AssetRecord.databaseTableName)") { db in
            try db.addColumnIfMissing(AssetRecord.Columns.isEnabled.name, .boolean, to: AssetRecord.databaseTableName) { $0.defaults(to: true) }
        }

        migrator.registerMigration("Add source to \(WalletRecord.databaseTableName)") { db in
            try db.addColumnIfMissing(WalletRecord.Columns.source.name, .text, to: WalletRecord.databaseTableName) { $0.defaults(to: WalletSource.create.rawValue) }
        }
        migrator.registerMigration("Add maxLeverage to \(PerpetualRecord.databaseTableName)") { db in
            try db.addColumnIfMissing(PerpetualRecord.Columns.maxLeverage.name, .integer, to: PerpetualRecord.databaseTableName) { $0.notNull().defaults(to: 1) }
        }

        migrator.registerMigration("Create \(RecentActivityRecord.databaseTableName)") { db in
            try db.createIfMissing(RecentActivityRecord.self)
        }

        migrator.registerMigration("Add pendingUnconfirmed to \(BalanceRecord.databaseTableName)") { db in
            try db.addColumnIfMissing(BalanceRecord.Columns.pendingUnconfirmed.name, .text, to: BalanceRecord.databaseTableName) { $0.defaults(to: "0") }
            try db.addColumnIfMissing(BalanceRecord.Columns.pendingUnconfirmedAmount.name, .double, to: BalanceRecord.databaseTableName) { $0.defaults(to: 0) }
        }

        migrator.registerMigration("Create \(SearchRecord.databaseTableName) and drop assets_search") { db in
            try db.dropTableIfExists("assets_search")
        }

        migrator.registerMigration("Create \(NotificationRecord.databaseTableName)") { db in
            try db.dropTableIfExists(NotificationRecord.databaseTableName)
            try NotificationRecord.create(db: db)
        }

        migrator.registerMigration("Add allTimeHigh/Low to \(PriceRecord.databaseTableName)") { db in
            try db.addColumnIfMissing(AssetMarketRecord.Columns.allTimeHigh.name, .double, to: PriceRecord.databaseTableName)
            try db.addColumnIfMissing(AssetMarketRecord.Columns.allTimeHighDate.name, .date, to: PriceRecord.databaseTableName)
            try db.addColumnIfMissing(AssetMarketRecord.Columns.allTimeHighChangePercentage.name, .double, to: PriceRecord.databaseTableName)
            try db.addColumnIfMissing(AssetMarketRecord.Columns.allTimeLow.name, .double, to: PriceRecord.databaseTableName)
            try db.addColumnIfMissing(AssetMarketRecord.Columns.allTimeLowDate.name, .date, to: PriceRecord.databaseTableName)
            try db.addColumnIfMissing(AssetMarketRecord.Columns.allTimeLowChangePercentage.name, .double, to: PriceRecord.databaseTableName)
        }

        migrator.registerMigration("Migrate wallet IDs to WalletIdentifier format") { db in
            try db.addColumnIfMissing(WalletRecord.Columns.externalId.name, .text, to: WalletRecord.databaseTableName)
            try WalletIdMigration.migrate(db: db)
        }

        migrator.registerMigration("Add hasImage to \(AssetRecord.databaseTableName)") { db in
            try db.addColumnIfMissing(AssetRecord.Columns.hasImage.name, .boolean, to: AssetRecord.databaseTableName) { $0.defaults(to: false) }
        }

        migrator.registerMigration("Create \(ContactRecord.databaseTableName) and \(ContactAddressRecord.databaseTableName)") { db in
            try db.createIfMissing(ContactRecord.self)
            try db.createIfMissing(ContactAddressRecord.self)
        }

        migrator.registerMigration("Add type to \(AddressRecord.databaseTableName)") { db in
            try db.addColumnIfMissing(AddressRecord.Columns.type.name, .text, to: AddressRecord.databaseTableName)
        }

        migrator.registerMigration("Add earn support") { db in
            try db.addColumnIfMissing(AssetRecord.Columns.isEarnable.name, .boolean, to: AssetRecord.databaseTableName) { $0.defaults(to: false) }
            try db.addColumnIfMissing(AssetRecord.Columns.earnApr.name, .double, to: AssetRecord.databaseTableName)
            try db.addColumnIfMissing(BalanceRecord.Columns.earn.name, .text, to: BalanceRecord.databaseTableName) { $0.defaults(to: "0") }
            try db.addColumnIfMissing(BalanceRecord.Columns.earnAmount.name, .double, to: BalanceRecord.databaseTableName) { $0.defaults(to: 0) }
            try Self.replaceTotalAmount(db)
        }

        migrator.registerMigration("Add providerType to stake_validators") { db in
            try db.addColumnIfMissing(StakeValidatorRecord.Columns.providerType.name, .text, to: StakeValidatorRecord.databaseTableName) {
                $0.defaults(to: StakeProviderType.stake.rawValue)
            }
        }

        migrator.registerMigration("Add status to \(AddressRecord.databaseTableName) and \(NFTCollectionRecord.databaseTableName)") { db in
            try db.addColumnIfMissing(AddressRecord.Columns.status.name, .text, to: AddressRecord.databaseTableName) {
                $0.notNull().defaults(to: VerificationStatus.unverified.rawValue)
            }
            try db.addColumnIfMissing(NFTCollectionRecord.Columns.status.name, .text, to: NFTCollectionRecord.databaseTableName) {
                $0.notNull().defaults(to: VerificationStatus.unverified.rawValue)
            }
        }

        migrator.registerMigration("Delete isVerified columns in \(NFTCollectionRecord.databaseTableName)") { db in
            if try db.columns(in: NFTCollectionRecord.databaseTableName).map(\.name).contains("isVerified") {
                try db.alter(table: NFTCollectionRecord.databaseTableName) {
                    $0.drop(column: "isVerified")
                }
            }
        }

        migrator.registerMigration("Create \(FiatTransactionRecord.databaseTableName)") { db in
            try db.createIfMissing(FiatTransactionRecord.self)
        }

        migrator.registerMigration("Recreate \(PerpetualRecord.databaseTableName)") { db in
            if try db.hasColumn(SearchRecord.Columns.perpetualId.name, in: SearchRecord.databaseTableName) {
                try db.execute(sql: "DELETE FROM \(SearchRecord.databaseTableName) WHERE \(SearchRecord.Columns.perpetualId.name) IS NOT NULL")
            }

            try db.dropTableIfExists(PerpetualPositionRecord.databaseTableName)
            try db.dropTableIfExists(PerpetualRecord.databaseTableName)

            try PerpetualRecord.create(db: db)
            try PerpetualPositionRecord.create(db: db)
        }

        migrator.registerMigration("Backfill type in \(AddressRecord.databaseTableName)") { db in
            try db.execute(
                sql: "UPDATE \(AddressRecord.databaseTableName) SET \(AddressRecord.Columns.type.name) = ? WHERE \(AddressRecord.Columns.type.name) IS NULL",
                arguments: [AddressType.address.rawValue],
            )
        }

        migrator.registerMigration("Clear NFT cache") { db in
            try Self.clearTables(
                db,
                tableNames: [
                    NFTAssetAssociationRecord.databaseTableName,
                    NFTAssetRecord.databaseTableName,
                    NFTCollectionRecord.databaseTableName,
                ],
            )
        }

        migrator.registerMigration("Create \(AssetListRecord.databaseTableName) and recreate \(SearchRecord.databaseTableName)") { db in
            try db.dropTableIfExists(SearchRecord.databaseTableName)
            try AssetListRecord.create(db: db)
            try SearchRecord.create(db: db)
        }

        migrator.registerMigration("Create \(SupportMessageRecord.databaseTableName)") { db in
            try SupportMessageRecord.create(db: db)
        }

        migrator.registerMigration("Add associations to \(AssetRecord.databaseTableName)") { db in
            try db.addColumnIfMissing(AssetRecord.Columns.associations.name, .jsonText, to: AssetRecord.databaseTableName) { $0.notNull().defaults(to: "[]") }
        }

        migrator.registerMigration("Add estimated confirmation to \(TransactionRecord.databaseTableName)") { db in
            try db.addColumnIfMissing(TransactionRecord.Columns.confirmationEtaSeconds.name, .integer, to: TransactionRecord.databaseTableName)
        }

        migrator.registerMigration("Add imageUrl to \(ContactRecord.databaseTableName)") { db in
            try db.addColumnIfMissing(ContactRecord.Columns.imageUrl.name, .text, to: ContactRecord.databaseTableName)
        }

        migrator.registerMigration("Add imageUrl to \(AddressRecord.databaseTableName)") { db in
            try db.addColumnIfMissing(AddressRecord.Columns.imageUrl.name, .text, to: AddressRecord.databaseTableName)
        }

        migrator.registerMigration("Recreate \(BannerRecord.databaseTableName) without chain") { db in
            try db.dropTableIfExists(BannerRecord.databaseTableName)
            try BannerRecord.create(db: db)
        }

        migrator.registerMigration("Drop node selection tables") { db in
            try db.dropTableIfExists("nodes_selected")
            try db.dropTableIfExists("nodes_selected_v1")
        }

        migrator.registerMigration("Recreate \(PriceAlertRecord.databaseTableName) with Core identifiers") { db in
            try db.dropTableIfExists(PriceAlertRecord.databaseTableName)
            try PriceAlertRecord.create(db: db)
        }

        migrator.registerMigration("Drop the market columns of \(PriceRecord.databaseTableName)") { db in
            let columns = try db.columns(in: PriceRecord.databaseTableName).map(\.name)
            let market = [
                AssetMarketRecord.Columns.marketCap,
                AssetMarketRecord.Columns.marketCapFdv,
                AssetMarketRecord.Columns.marketCapRank,
                AssetMarketRecord.Columns.totalVolume,
                AssetMarketRecord.Columns.circulatingSupply,
                AssetMarketRecord.Columns.totalSupply,
                AssetMarketRecord.Columns.maxSupply,
                AssetMarketRecord.Columns.allTimeHigh,
                AssetMarketRecord.Columns.allTimeHighDate,
                AssetMarketRecord.Columns.allTimeHighChangePercentage,
                AssetMarketRecord.Columns.allTimeLow,
                AssetMarketRecord.Columns.allTimeLowDate,
                AssetMarketRecord.Columns.allTimeLowChangePercentage,
            ].map(\.name).filter { columns.contains($0) }
            guard !market.isEmpty else { return }
            try db.alter(table: PriceRecord.databaseTableName) { table in
                for name in market {
                    table.drop(column: name)
                }
            }
        }

        migrator.registerMigration("Drop the unread lastUsedAt column of \(BalanceRecord.databaseTableName)") { db in
            guard try db.columns(in: BalanceRecord.databaseTableName).contains(where: { $0.name == "lastUsedAt" }) else { return }
            try db.alter(table: BalanceRecord.databaseTableName) {
                $0.drop(column: "lastUsedAt")
            }
        }

        migrator.registerMigration("Delete \(FiatRateRecord.databaseTableName) rows with an unknown currency") { db in
            let known = Currency.allCases.map { "'\($0.rawValue)'" }.joined(separator: ",")
            try db.execute(sql: "DELETE FROM \(FiatRateRecord.databaseTableName) WHERE \(FiatRateRecord.Columns.symbol.name) NOT IN (\(known))")
        }

        migrator.registerMigration("Index \(TransactionRecord.databaseTableName) by wallet and date") { db in
            guard try db.hasColumn(TransactionRecord.Columns.date.name, in: TransactionRecord.databaseTableName) else { return }
            try db.create(indexOn: TransactionRecord.databaseTableName, columns: [TransactionRecord.Columns.walletId.name, TransactionRecord.Columns.date.name], options: .ifNotExists)
            try db.drop(indexOn: TransactionRecord.databaseTableName, columns: [TransactionRecord.Columns.walletId.name])
            try db.drop(indexOn: TransactionRecord.databaseTableName, columns: [TransactionRecord.Columns.date.name])
        }

        try migrator.migrate(dbQueue)
    }
}

private extension Database {
    func hasColumn(_ column: String, in table: String) throws -> Bool {
        try tableExists(table) && columns(in: table).contains { $0.name == column }
    }

    func dropTableIfExists(_ table: String) throws {
        guard try tableExists(table) else { return }
        try drop(table: table)
    }

    func createIfMissing(_ record: any CreateTable.Type) throws {
        guard try !tableExists(record.databaseTableName) else { return }
        try record.create(db: self)
    }

    func addColumnIfMissing(_ column: String, _ type: Database.ColumnType, to table: String, _ configure: (ColumnDefinition) -> Void = { _ in }) throws {
        guard try tableExists(table), try !columns(in: table).contains(where: { $0.name == column }) else { return }
        try alter(table: table) { configure($0.add(column: column, type)) }
    }
}
