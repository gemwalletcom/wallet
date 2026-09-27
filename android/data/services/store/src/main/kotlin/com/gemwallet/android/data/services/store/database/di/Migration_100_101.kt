package com.gemwallet.android.data.services.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_100_101 : Migration(100, 101) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL("ALTER TABLE `nft_assets` ADD COLUMN `resource_url` TEXT NOT NULL DEFAULT ''")
        db.execSQL("ALTER TABLE `nft_assets` ADD COLUMN `resource_mime_type` TEXT NOT NULL DEFAULT ''")
    }
}
