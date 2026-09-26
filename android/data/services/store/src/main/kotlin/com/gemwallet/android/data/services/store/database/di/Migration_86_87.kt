package com.gemwallet.android.data.services.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_86_87 : Migration(86, 87) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL("ALTER TABLE `prices` ADD COLUMN `updatedAt` INTEGER")
    }
}
