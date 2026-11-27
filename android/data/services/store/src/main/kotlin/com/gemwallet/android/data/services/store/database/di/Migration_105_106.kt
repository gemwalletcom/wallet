package com.gemwallet.android.data.services.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_105_106 : Migration(105, 106) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL("UPDATE transactions SET fee = '0' WHERE fee LIKE '-%'")
    }
}
