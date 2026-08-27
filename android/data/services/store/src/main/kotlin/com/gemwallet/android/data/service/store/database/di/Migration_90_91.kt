package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_90_91 : Migration(90, 91) {
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
                FOREIGN KEY(`chain`) REFERENCES `asset`(`id`) ON UPDATE CASCADE ON DELETE CASCADE,
                FOREIGN KEY(`wallet_id`) REFERENCES `wallets`(`id`) ON UPDATE CASCADE ON DELETE CASCADE
            )
            """.trimIndent(),
        )
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_banners_event` ON `banners` (`event`)")
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_banners_wallet_id` ON `banners` (`wallet_id`)")
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_banners_chain` ON `banners` (`chain`)")
    }
}
