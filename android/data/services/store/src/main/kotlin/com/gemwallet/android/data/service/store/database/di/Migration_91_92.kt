package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_91_92 : Migration(91, 92) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL("CREATE TABLE IF NOT EXISTS `transactions_new` (`id` TEXT NOT NULL, `walletId` TEXT NOT NULL, `hash` TEXT NOT NULL, `assetId` TEXT NOT NULL, `feeAssetId` TEXT NOT NULL, `owner` TEXT NOT NULL, `recipient` TEXT NOT NULL, `contract` TEXT, `metadata` TEXT, `state` TEXT NOT NULL, `type` TEXT NOT NULL, `blockNumber` TEXT NOT NULL, `sequence` TEXT NOT NULL, `fee` TEXT NOT NULL, `value` TEXT NOT NULL, `payload` TEXT, `direction` TEXT NOT NULL, `createdAt` INTEGER NOT NULL, `updatedAt` INTEGER NOT NULL, `estimatedConfirmationInSeconds` INTEGER, `recordId` INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, FOREIGN KEY(`walletId`) REFERENCES `wallets`(`id`) ON UPDATE CASCADE ON DELETE CASCADE )")
        db.execSQL("INSERT INTO transactions_new (`id`, `walletId`, `hash`, `assetId`, `feeAssetId`, `owner`, `recipient`, `contract`, `metadata`, `state`, `type`, `blockNumber`, `sequence`, `fee`, `value`, `payload`, `direction`, `createdAt`, `updatedAt`, `estimatedConfirmationInSeconds`) SELECT `id`, `walletId`, `hash`, `assetId`, `feeAssetId`, `owner`, `recipient`, `contract`, `metadata`, `state`, `type`, `blockNumber`, `sequence`, `fee`, `value`, `payload`, `direction`, `createdAt`, `updatedAt`, `estimatedConfirmationInSeconds` FROM transactions")
        db.execSQL("DROP TABLE transactions")
        db.execSQL("ALTER TABLE transactions_new RENAME TO transactions")
        db.execSQL("CREATE INDEX index_transactions_walletId ON transactions (walletId)")
        db.execSQL("CREATE UNIQUE INDEX index_transactions_walletId_id ON transactions (walletId, id)")
    }
}
