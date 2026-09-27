package com.gemwallet.android.data.services.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_94_95 : Migration(94, 95) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL("ALTER TABLE `asset` ADD COLUMN `has_image` INTEGER NOT NULL DEFAULT 0")
    }
}
