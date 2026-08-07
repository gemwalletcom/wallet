package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_83_84 : Migration(83, 84) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL("ALTER TABLE `asset` ADD COLUMN `associations` TEXT NOT NULL DEFAULT '[]'")
    }
}
