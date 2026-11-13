package com.gemwallet.android.data.services.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_104_105 : Migration(104, 105) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.removeDeprecatedChains(listOf("sei", "blast"))
    }
}
