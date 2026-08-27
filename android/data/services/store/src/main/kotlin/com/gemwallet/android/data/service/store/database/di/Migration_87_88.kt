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
    }
}
