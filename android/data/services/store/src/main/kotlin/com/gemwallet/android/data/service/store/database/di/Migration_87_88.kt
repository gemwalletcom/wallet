package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_87_88 : Migration(87, 88) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL("DROP TABLE IF EXISTS `banners`")
        db.execSQL(
            """
            CREATE TABLE IF NOT EXISTS `banners` (
                `id` TEXT NOT NULL,
                `wallet_id` TEXT,
                `asset_id` TEXT,
                `chain` TEXT,
                `state` TEXT NOT NULL,
                `event` TEXT NOT NULL,
                PRIMARY KEY(`id`),
                FOREIGN KEY(`chain`) REFERENCES `asset`(`id`) ON UPDATE CASCADE ON DELETE CASCADE
            )
            """.trimIndent(),
        )
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_banners_event` ON `banners` (`event`)")
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_banners_wallet_id` ON `banners` (`wallet_id`)")
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_banners_chain` ON `banners` (`chain`)")
        db.execSQL("ALTER TABLE `asset` ADD COLUMN `is_earn_enabled` INTEGER NOT NULL DEFAULT 0")
        db.execSQL("ALTER TABLE `asset` ADD COLUMN `earn_apr` REAL")
        db.execSQL("ALTER TABLE `balances` ADD COLUMN `pending_unconfirmed` TEXT NOT NULL DEFAULT '0'")
        db.execSQL("ALTER TABLE `balances` ADD COLUMN `pending_unconfirmed_amount` REAL NOT NULL DEFAULT 0.0")
        db.execSQL("ALTER TABLE `balances` ADD COLUMN `earn` TEXT NOT NULL DEFAULT '0'")
        db.execSQL("ALTER TABLE `balances` ADD COLUMN `earn_amount` REAL NOT NULL DEFAULT 0.0")
        db.execSQL("DROP TABLE IF EXISTS `price_alerts`")
        db.execSQL(
            """
            CREATE TABLE IF NOT EXISTS `price_alerts` (
                `id` TEXT NOT NULL,
                `assetId` TEXT NOT NULL,
                `currency` TEXT NOT NULL,
                `price` REAL,
                `pricePercentChange` REAL,
                `priceDirection` TEXT,
                `lastNotifiedAt` INTEGER,
                PRIMARY KEY(`id`)
            )
            """.trimIndent(),
        )
    }
}
