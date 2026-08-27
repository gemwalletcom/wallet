package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_88_89 : Migration(88, 89) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL("ALTER TABLE `asset` ADD COLUMN `is_earn_enabled` INTEGER NOT NULL DEFAULT 0")
        db.execSQL("ALTER TABLE `asset` ADD COLUMN `earn_apr` REAL")
    }
}
