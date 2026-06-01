package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_75_76 : Migration(75, 76) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL(
            """
                CREATE TABLE IF NOT EXISTS `in_app_notifications` (
                    `id` TEXT NOT NULL,
                    `wallet_id` TEXT NOT NULL,
                    `read_at` INTEGER,
                    `created_at` INTEGER NOT NULL,
                    `item` TEXT NOT NULL,
                    PRIMARY KEY(`id`),
                    FOREIGN KEY(`wallet_id`) REFERENCES `wallets`(`id`) ON UPDATE CASCADE ON DELETE CASCADE
                )
            """
        )
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_in_app_notifications_wallet_id` ON `in_app_notifications` (`wallet_id`)")
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_in_app_notifications_created_at` ON `in_app_notifications` (`created_at`)")
    }
}
