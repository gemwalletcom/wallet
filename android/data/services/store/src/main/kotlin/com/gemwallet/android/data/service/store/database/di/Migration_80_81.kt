package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_80_81 : Migration(80, 81) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL("DROP TABLE IF EXISTS `assets_priority`")
        db.execSQL(
            "CREATE TABLE IF NOT EXISTS `search` (" +
                "`id` INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, " +
                "`query` TEXT NOT NULL, " +
                "`assetId` TEXT, " +
                "`perpetualId` TEXT, " +
                "`priority` INTEGER NOT NULL, " +
                "FOREIGN KEY(`assetId`) REFERENCES `asset`(`id`) ON UPDATE NO ACTION ON DELETE CASCADE , " +
                "FOREIGN KEY(`perpetualId`) REFERENCES `perpetuals`(`id`) ON UPDATE NO ACTION ON DELETE CASCADE )"
        )
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_search_query` ON `search` (`query`)")
        db.execSQL("CREATE UNIQUE INDEX IF NOT EXISTS `index_search_assetId_query` ON `search` (`assetId`, `query`)")
        db.execSQL("CREATE UNIQUE INDEX IF NOT EXISTS `index_search_perpetualId_query` ON `search` (`perpetualId`, `query`)")
    }
}
