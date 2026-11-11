package com.gemwallet.android.data.services.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_103_104 : Migration(103, 104) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL("ALTER TABLE `asset` ADD COLUMN `has_candles` INTEGER NOT NULL DEFAULT 0")
    }
}
