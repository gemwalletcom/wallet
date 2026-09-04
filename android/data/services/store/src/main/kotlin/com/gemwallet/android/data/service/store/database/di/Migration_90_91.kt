package com.gemwallet.android.data.service.store.database.di

import android.content.Context
import androidx.core.content.edit
import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

class Migration_90_91(context: Context) : Migration(90, 91) {

    private val preferences = context.getSharedPreferences(PREFERENCES, Context.MODE_PRIVATE)

    override fun migrate(db: SupportSQLiteDatabase) {
        var walletId: String? = null
        var currency: String? = null
        db.query("SELECT wallet_id, currency FROM session WHERE id = 1").use { cursor ->
            if (cursor.moveToNext()) {
                walletId = cursor.getString(0)
                currency = cursor.getString(1)
            }
        }
        val storedCurrency = currency.takeUnless { it.isNullOrEmpty() || preferences.contains(CURRENCY) }

        preferences.edit(commit = true) {
            walletId.takeUnless { it.isNullOrEmpty() }?.let { putString(CURRENT_WALLET_ID, it) }
            storedCurrency?.let { putString(CURRENCY, it) }
        }
        db.execSQL("DROP TABLE session")
    }

    private companion object {
        const val PREFERENCES = "gemstone_preferences"
        const val CURRENT_WALLET_ID = "current_wallet_id"
        const val CURRENCY = "currency"
    }
}
