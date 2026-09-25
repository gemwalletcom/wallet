package com.gemwallet.android.data.services.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase
import com.wallet.core.primitives.Currency

object Migration_92_93 : Migration(92, 93) {
    override fun migrate(db: SupportSQLiteDatabase) {
        val known = Currency.entries.joinToString(",") { "'${it.string}'" }
        db.execSQL("DELETE FROM currency_rates WHERE currency NOT IN ($known)")
    }
}
