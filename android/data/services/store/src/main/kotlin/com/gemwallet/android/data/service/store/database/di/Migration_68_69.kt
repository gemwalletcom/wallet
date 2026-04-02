package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_68_69 : Migration(68, 69) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL(
            """CREATE TABLE IF NOT EXISTS `fiat_transactions` (
                `id` TEXT NOT NULL,
                `walletId` TEXT NOT NULL,
                `assetId` TEXT NOT NULL,
                `transactionType` TEXT NOT NULL,
                `provider` TEXT NOT NULL,
                `status` TEXT NOT NULL,
                `fiatAmount` REAL NOT NULL,
                `fiatCurrency` TEXT NOT NULL,
                `value` TEXT NOT NULL,
                `createdAt` INTEGER NOT NULL,
                `detailsUrl` TEXT,
                PRIMARY KEY(`id`, `walletId`),
                FOREIGN KEY(`walletId`) REFERENCES `wallets`(`id`) ON UPDATE CASCADE ON DELETE CASCADE,
                FOREIGN KEY(`assetId`) REFERENCES `asset`(`id`) ON UPDATE CASCADE ON DELETE CASCADE
            )"""
        )
    }
}
