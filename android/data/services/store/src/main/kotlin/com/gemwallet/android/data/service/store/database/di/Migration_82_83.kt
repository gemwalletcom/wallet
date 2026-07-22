package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_82_83 : Migration(82, 83) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL(
            "CREATE TABLE IF NOT EXISTS `transactions_new` (" +
                "`id` TEXT NOT NULL, " +
                "`walletId` TEXT NOT NULL, " +
                "`hash` TEXT NOT NULL, " +
                "`assetId` TEXT NOT NULL, " +
                "`feeAssetId` TEXT NOT NULL, " +
                "`owner` TEXT NOT NULL, " +
                "`recipient` TEXT NOT NULL, " +
                "`contract` TEXT, " +
                "`metadata` TEXT, " +
                "`state` TEXT NOT NULL, " +
                "`type` TEXT NOT NULL, " +
                "`blockNumber` TEXT NOT NULL, " +
                "`sequence` TEXT NOT NULL, " +
                "`fee` TEXT NOT NULL, " +
                "`value` TEXT NOT NULL, " +
                "`payload` TEXT, " +
                "`direction` TEXT NOT NULL, " +
                "`createdAt` INTEGER NOT NULL, " +
                "`updatedAt` INTEGER NOT NULL, " +
                "PRIMARY KEY(`id`, `walletId`), " +
                "FOREIGN KEY(`walletId`) REFERENCES `wallets`(`id`) ON UPDATE CASCADE ON DELETE CASCADE)"
        )
        db.execSQL(
            "INSERT INTO `transactions_new` (" +
                "`id`, `walletId`, `hash`, `assetId`, `feeAssetId`, `owner`, `recipient`, `contract`, " +
                "`metadata`, `state`, `type`, `blockNumber`, `sequence`, `fee`, `value`, `payload`, " +
                "`direction`, `createdAt`, `updatedAt`) " +
                "SELECT `id`, `walletId`, `hash`, `assetId`, `feeAssetId`, `owner`, `recipient`, `contract`, " +
                "`metadata`, `state`, `type`, `blockNumber`, `sequence`, `fee`, `value`, `payload`, " +
                "`direction`, `createdAt`, `updatedAt` FROM `transactions` " +
                "WHERE `walletId` IN (SELECT `id` FROM `wallets`)"
        )
        db.execSQL("DROP TABLE `transactions`")
        db.execSQL("ALTER TABLE `transactions_new` RENAME TO `transactions`")
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_transactions_walletId` ON `transactions` (`walletId`)")
    }
}
