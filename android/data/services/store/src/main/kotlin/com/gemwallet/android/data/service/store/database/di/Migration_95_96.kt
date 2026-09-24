package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_95_96 : Migration(95, 96) {
    private val keptColumns = listOf(
        "asset_id",
        "wallet_id",
        "available",
        "available_amount",
        "frozen",
        "frozen_amount",
        "locked",
        "locked_amount",
        "staked",
        "staked_amount",
        "pending",
        "pending_amount",
        "rewards",
        "rewards_amount",
        "reserved",
        "reserved_amount",
        "withdrawable",
        "withdrawableAmount",
        "pending_unconfirmed",
        "pending_unconfirmed_amount",
        "earn",
        "earn_amount",
        "total_amount",
        "is_active",
        "is_pinned",
        "is_visible",
        "updated_at",
    ).joinToString { "`$it`" }

    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL(
            "CREATE TABLE IF NOT EXISTS `_new_balances` (" +
                "`asset_id` TEXT NOT NULL, " +
                "`wallet_id` TEXT NOT NULL, " +
                "`available` TEXT NOT NULL, " +
                "`available_amount` REAL NOT NULL, " +
                "`frozen` TEXT NOT NULL, " +
                "`frozen_amount` REAL NOT NULL, " +
                "`locked` TEXT NOT NULL, " +
                "`locked_amount` REAL NOT NULL, " +
                "`staked` TEXT NOT NULL, " +
                "`staked_amount` REAL NOT NULL, " +
                "`pending` TEXT NOT NULL, " +
                "`pending_amount` REAL NOT NULL, " +
                "`rewards` TEXT NOT NULL, " +
                "`rewards_amount` REAL NOT NULL, " +
                "`reserved` TEXT NOT NULL, " +
                "`reserved_amount` REAL NOT NULL, " +
                "`withdrawable` TEXT NOT NULL, " +
                "`withdrawableAmount` REAL NOT NULL, " +
                "`pending_unconfirmed` TEXT NOT NULL DEFAULT '0', " +
                "`pending_unconfirmed_amount` REAL NOT NULL DEFAULT 0.0, " +
                "`earn` TEXT NOT NULL DEFAULT '0', " +
                "`earn_amount` REAL NOT NULL DEFAULT 0.0, " +
                "`total_amount` REAL NOT NULL, " +
                "`is_active` INTEGER NOT NULL, " +
                "`is_pinned` INTEGER NOT NULL, " +
                "`is_visible` INTEGER NOT NULL, " +
                "`metadata` TEXT, " +
                "`updated_at` INTEGER, " +
                "PRIMARY KEY(`asset_id`, `wallet_id`), " +
                "FOREIGN KEY(`asset_id`) REFERENCES `asset`(`id`) ON UPDATE CASCADE ON DELETE CASCADE, " +
                "FOREIGN KEY(`wallet_id`) REFERENCES `wallets`(`id`) ON UPDATE CASCADE ON DELETE CASCADE)",
        )
        db.execSQL("INSERT INTO `_new_balances` ($keptColumns) SELECT $keptColumns FROM `balances`")
        db.execSQL("DROP TABLE `balances`")
        db.execSQL("ALTER TABLE `_new_balances` RENAME TO `balances`")
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_balances_wallet_id` ON `balances` (`wallet_id`)")
    }
}
