package com.gemwallet.android.data.services.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_105_106 : Migration(105, 106) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL(
            "CREATE TABLE IF NOT EXISTS `lightning_leaves` (`wallet_id` TEXT NOT NULL, `id` TEXT NOT NULL, `data` TEXT NOT NULL, PRIMARY KEY(`wallet_id`, `id`), " +
                "FOREIGN KEY(`wallet_id`) REFERENCES `wallets`(`id`) ON UPDATE CASCADE ON DELETE CASCADE )",
        )
        db.execSQL(
            "CREATE TABLE IF NOT EXISTS `lightning_pending_sends` (`wallet_id` TEXT NOT NULL, `transfer_id` TEXT NOT NULL, `invoice` TEXT NOT NULL, `amount_sats` INTEGER, " +
                "`expires_at` INTEGER NOT NULL, PRIMARY KEY(`wallet_id`, `transfer_id`), FOREIGN KEY(`wallet_id`) REFERENCES `wallets`(`id`) ON UPDATE CASCADE ON DELETE CASCADE )",
        )
    }
}
