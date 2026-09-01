package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_89_90 : Migration(89, 90) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL("DELETE FROM recent_assets")
    }
}
