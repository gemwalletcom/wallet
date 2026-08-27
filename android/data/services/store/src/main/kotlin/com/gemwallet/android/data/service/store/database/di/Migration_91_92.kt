package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_91_92 : Migration(91, 92) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL("DROP TABLE IF EXISTS `nodes`")
        db.execSQL(
            """
            CREATE TABLE IF NOT EXISTS `nodes` (
                `url` TEXT NOT NULL,
                `status` TEXT NOT NULL,
                `priority` INTEGER NOT NULL,
                `chain` TEXT NOT NULL,
                PRIMARY KEY(`chain`, `url`),
                FOREIGN KEY(`chain`) REFERENCES `asset`(`id`) ON UPDATE CASCADE ON DELETE CASCADE
            )
            """.trimIndent(),
        )
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_nodes_chain` ON `nodes` (`chain`)")
    }
}
