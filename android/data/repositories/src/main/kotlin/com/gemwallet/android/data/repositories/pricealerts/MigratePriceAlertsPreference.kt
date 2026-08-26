package com.gemwallet.android.data.repositories.pricealerts

import android.content.Context
import androidx.core.content.edit
import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.preferencesDataStore
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemPreferencesService

class MigratePriceAlertsPreference(
    private val context: Context,
    private val preferencesService: GemPreferencesService,
) {
    private val Context.dataStore by preferencesDataStore(name = "price_alerts")
    private val enabledKey = booleanPreferencesKey("price_alerts_enabled")

    suspend operator fun invoke() = withContext(Dispatchers.IO) {
        val legacyStore = context.getSharedPreferences("price-alerts", Context.MODE_PRIVATE)
        val enabled = context.dataStore.data.first()[enabledKey]
            ?: legacyStore.takeIf { it.contains(LEGACY_KEY) }?.getBoolean(LEGACY_KEY, false)
            ?: return@withContext
        preferencesService.setPriceAlertsEnabled(enabled)
        context.dataStore.edit { it.remove(enabledKey) }
        legacyStore.edit(commit = true) { remove(LEGACY_KEY) }
    }

    private companion object {
        const val LEGACY_KEY = "price_alerts_enabled-"
    }
}
