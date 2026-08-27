package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_89_90 : Migration(89, 90) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL("ALTER TABLE `balances` ADD COLUMN `pending_unconfirmed` TEXT NOT NULL DEFAULT '0'")
        db.execSQL("ALTER TABLE `balances` ADD COLUMN `pending_unconfirmed_amount` REAL NOT NULL DEFAULT 0.0")
        db.execSQL("ALTER TABLE `balances` ADD COLUMN `earn` TEXT NOT NULL DEFAULT '0'")
        db.execSQL("ALTER TABLE `balances` ADD COLUMN `earn_amount` REAL NOT NULL DEFAULT 0.0")
    }
}
