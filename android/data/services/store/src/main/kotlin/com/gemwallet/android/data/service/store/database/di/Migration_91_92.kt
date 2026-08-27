package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_91_92 : Migration(91, 92) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL("CREATE TABLE IF NOT EXISTS `session_new` (`id` INTEGER NOT NULL, `wallet_id` TEXT, `currency` TEXT NOT NULL, PRIMARY KEY(`id`))")
        db.execSQL("INSERT INTO `session_new` (`id`, `wallet_id`, `currency`) SELECT `id`, `wallet_id`, `currency` FROM `session`")
        db.execSQL("DROP TABLE `session`")
        db.execSQL("ALTER TABLE `session_new` RENAME TO `session`")

        db.execSQL(
            """
            CREATE TABLE IF NOT EXISTS `nodes_new` (
                `url` TEXT NOT NULL,
                `status` TEXT NOT NULL,
                `priority` INTEGER NOT NULL,
                `chain` TEXT NOT NULL,
                PRIMARY KEY(`chain`, `url`),
                FOREIGN KEY(`chain`) REFERENCES `asset`(`id`) ON UPDATE CASCADE ON DELETE CASCADE
            )
            """.trimIndent(),
        )
        db.execSQL("INSERT INTO `nodes_new` (`url`, `status`, `priority`, `chain`) SELECT `url`, `status`, `priority`, `chain` FROM `nodes`")
        db.execSQL("DROP TABLE `nodes`")
        db.execSQL("ALTER TABLE `nodes_new` RENAME TO `nodes`")
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_nodes_chain` ON `nodes` (`chain`)")
    }
}
