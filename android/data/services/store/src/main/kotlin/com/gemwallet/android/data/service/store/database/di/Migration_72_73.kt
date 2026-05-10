package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_72_73 : Migration(72, 73) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL(
            """
                CREATE TABLE IF NOT EXISTS `addresses` (
                    `chain` TEXT NOT NULL,
                    `address` TEXT NOT NULL,
                    `walletId` TEXT,
                    `name` TEXT NOT NULL,
                    `type` TEXT NOT NULL,
                    `status` TEXT NOT NULL,
                    PRIMARY KEY(`chain`, `address`),
                    FOREIGN KEY(`chain`) REFERENCES `asset`(`id`) ON UPDATE CASCADE ON DELETE CASCADE,
                    FOREIGN KEY(`walletId`) REFERENCES `wallets`(`id`) ON UPDATE CASCADE ON DELETE CASCADE
                )
            """
        )
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_addresses_chain` ON `addresses` (`chain`)")
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_addresses_walletId` ON `addresses` (`walletId`)")
    }
}
