package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_81_82 : Migration(81, 82) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL(
            "CREATE TABLE IF NOT EXISTS `asset_lists` (" +
                "`id` TEXT NOT NULL, " +
                "`name` TEXT NOT NULL, " +
                "`count` INTEGER NOT NULL, " +
                "PRIMARY KEY(`id`))"
        )
        db.execSQL("DROP TABLE IF EXISTS `search`")
        db.execSQL(
            "CREATE TABLE IF NOT EXISTS `search` (" +
                "`id` INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, " +
                "`query` TEXT NOT NULL, " +
                "`assetId` TEXT, " +
                "`perpetualId` TEXT, " +
                "`listId` TEXT, " +
                "`priority` INTEGER NOT NULL, " +
                "FOREIGN KEY(`assetId`) REFERENCES `asset`(`id`) ON UPDATE NO ACTION ON DELETE CASCADE , " +
                "FOREIGN KEY(`perpetualId`) REFERENCES `perpetuals`(`id`) ON UPDATE NO ACTION ON DELETE CASCADE , " +
                "FOREIGN KEY(`listId`) REFERENCES `asset_lists`(`id`) ON UPDATE NO ACTION ON DELETE CASCADE )"
        )
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_search_query` ON `search` (`query`)")
        db.execSQL("CREATE UNIQUE INDEX IF NOT EXISTS `index_search_assetId_query` ON `search` (`assetId`, `query`)")
        db.execSQL("CREATE UNIQUE INDEX IF NOT EXISTS `index_search_perpetualId_query` ON `search` (`perpetualId`, `query`)")
        db.execSQL("CREATE UNIQUE INDEX IF NOT EXISTS `index_search_listId_query` ON `search` (`listId`, `query`)")
    }
}
