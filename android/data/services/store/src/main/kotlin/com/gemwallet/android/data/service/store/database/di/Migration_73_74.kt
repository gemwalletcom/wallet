package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_73_74 : Migration(73, 74) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL("DROP TABLE IF EXISTS `perpetual_position`")
        db.execSQL("DROP TABLE IF EXISTS `perpetual_balance`")
        db.execSQL("DROP TABLE IF EXISTS `perpetual_metadata`")
        db.execSQL("DROP TABLE IF EXISTS `perpetual`")
        db.execSQL("DROP TABLE IF EXISTS `perpetual_asset`")

        db.execSQL("ALTER TABLE `balances` ADD COLUMN `withdrawable` TEXT NOT NULL DEFAULT '0'")
        db.execSQL("ALTER TABLE `balances` ADD COLUMN `withdrawableAmount` REAL NOT NULL DEFAULT 0")

        db.execSQL(
            """
                CREATE TABLE IF NOT EXISTS `perpetuals` (
                    `id` TEXT NOT NULL,
                    `name` TEXT NOT NULL,
                    `provider` TEXT NOT NULL,
                    `assetId` TEXT NOT NULL,
                    `identifier` TEXT NOT NULL,
                    `price` REAL NOT NULL,
                    `pricePercentChange24h` REAL NOT NULL,
                    `openInterest` REAL NOT NULL,
                    `volume24h` REAL NOT NULL,
                    `funding` REAL NOT NULL,
                    `maxLeverage` INTEGER NOT NULL,
                    `isIsolatedOnly` INTEGER NOT NULL DEFAULT 0,
                    `isPinned` INTEGER NOT NULL DEFAULT 0,
                    PRIMARY KEY(`id`),
                    FOREIGN KEY(`assetId`) REFERENCES `asset`(`id`) ON UPDATE CASCADE ON DELETE CASCADE
                )
            """
        )
        db.execSQL("CREATE INDEX IF NOT EXISTS `perpetuals_asset_id_idx` ON `perpetuals` (`assetId`)")

        db.execSQL(
            """
                CREATE TABLE IF NOT EXISTS `perpetuals_positions` (
                    `id` TEXT NOT NULL,
                    `walletId` TEXT NOT NULL,
                    `perpetualId` TEXT NOT NULL,
                    `assetId` TEXT NOT NULL,
                    `size` REAL NOT NULL,
                    `sizeValue` REAL NOT NULL,
                    `leverage` INTEGER NOT NULL,
                    `entryPrice` REAL,
                    `liquidationPrice` REAL,
                    `marginType` TEXT NOT NULL,
                    `direction` TEXT NOT NULL,
                    `marginAmount` REAL NOT NULL,
                    `takeProfitPrice` REAL,
                    `takeProfitType` TEXT,
                    `takeProfitOrderId` TEXT,
                    `stopLossPrice` REAL,
                    `stopLossType` TEXT,
                    `stopLossOrderId` TEXT,
                    `pnl` REAL NOT NULL,
                    `funding` REAL,
                    `updatedAt` INTEGER NOT NULL,
                    PRIMARY KEY(`id`, `walletId`),
                    FOREIGN KEY(`walletId`) REFERENCES `wallets`(`id`) ON UPDATE CASCADE ON DELETE CASCADE,
                    FOREIGN KEY(`perpetualId`) REFERENCES `perpetuals`(`id`) ON UPDATE CASCADE ON DELETE CASCADE,
                    FOREIGN KEY(`assetId`) REFERENCES `asset`(`id`) ON UPDATE CASCADE ON DELETE CASCADE
                )
            """
        )
        db.execSQL("CREATE INDEX IF NOT EXISTS `perpetuals_positions_wallet_id_idx` ON `perpetuals_positions` (`walletId`)")
        db.execSQL("CREATE INDEX IF NOT EXISTS `perpetuals_positions_perpetual_id_idx` ON `perpetuals_positions` (`perpetualId`)")
        db.execSQL("CREATE INDEX IF NOT EXISTS `perpetuals_positions_asset_id_idx` ON `perpetuals_positions` (`assetId`)")
    }
}
