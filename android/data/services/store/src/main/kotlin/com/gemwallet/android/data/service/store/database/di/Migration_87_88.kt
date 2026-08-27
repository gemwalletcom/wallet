package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_87_88 : Migration(87, 88) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL(
            """
                CREATE TABLE IF NOT EXISTS `banners_new` (
                    `id` TEXT NOT NULL,
                    `wallet_id` TEXT,
                    `asset_id` TEXT,
                    `chain` TEXT,
                    `state` TEXT NOT NULL,
                    `event` TEXT NOT NULL,
                    PRIMARY KEY(`id`),
                    FOREIGN KEY(`chain`) REFERENCES `asset`(`id`) ON UPDATE CASCADE ON DELETE CASCADE
                )
            """
        )
        db.execSQL(
            """
                INSERT OR IGNORE INTO `banners_new` (`id`, `wallet_id`, `asset_id`, `chain`, `state`, `event`)
                SELECT
                    CASE WHEN wallet_id <> '' THEN wallet_id || '_' ELSE '' END
                        || CASE WHEN asset_id <> '' THEN asset_id || '_' ELSE '' END
                        || CASE WHEN chain IS NOT NULL THEN chain || '_' ELSE '' END
                        || lower(substr(event, 1, 1)) || substr(event, 2),
                    NULLIF(wallet_id, ''),
                    NULLIF(asset_id, ''),
                    chain,
                    state,
                    event
                FROM banners
            """
        )
        db.execSQL("DROP TABLE `banners`")
        db.execSQL("ALTER TABLE `banners_new` RENAME TO `banners`")
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_banners_event` ON `banners` (`event`)")
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_banners_wallet_id` ON `banners` (`wallet_id`)")
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_banners_chain` ON `banners` (`chain`)")
    }
}
