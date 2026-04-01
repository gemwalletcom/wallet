package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_62_63 : Migration(62, 63) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL("ALTER TABLE price_alerts ADD COLUMN lastNotifiedAt INTEGER")
        db.execSQL(
            """CREATE TABLE IF NOT EXISTS `nft_collection_link` (
                `collection_id` TEXT NOT NULL,
                `name` TEXT NOT NULL,
                `url` TEXT NOT NULL,
                PRIMARY KEY(`collection_id`, `name`),
                FOREIGN KEY(`collection_id`) REFERENCES `nft_collection`(`id`) ON UPDATE NO ACTION ON DELETE CASCADE
            )"""
        )
    }
}