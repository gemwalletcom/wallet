package com.gemwallet.android.data.adapters.gemstone

import android.content.Context
import android.content.SharedPreferences
import androidx.core.content.edit
import uniffi.gemstone.GemWalletPreferencesStore

class GemstoneWalletPreferencesStore(
    private val context: Context,
) : GemWalletPreferencesStore {
    override fun get(walletId: String, key: String): String? = preferences(walletId).getString(key, null)

    override fun set(walletId: String, key: String, value: String) = preferences(walletId).edit { putString(key, value) }

    override fun deletePreferences(walletId: String) = preferences(walletId).edit { clear() }

    private fun preferences(walletId: String): SharedPreferences =
        context.getSharedPreferences("wallet_preferences_${walletId}_v2", Context.MODE_PRIVATE)
}
