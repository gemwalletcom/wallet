package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_80_81 : Migration(80, 81) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL("DROP TABLE IF EXISTS `assets_priority`")
        db.execSQL(
            "CREATE TABLE IF NOT EXISTS `search` (" +
                "`query` TEXT NOT NULL, " +
                "`type` TEXT NOT NULL, " +
                "`item_id` TEXT NOT NULL, " +
                "`priority` INTEGER NOT NULL, " +
                "PRIMARY KEY(`query`, `type`, `item_id`))"
        )
    }
}
