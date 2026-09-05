package com.gemwallet.android.data.services.gemstone.stores

import android.content.SharedPreferences
import androidx.core.content.edit
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.distinctUntilChanged
import uniffi.gemstone.GemPreferencesStore

class GemstonePreferencesStore(
    private val sharedPreferences: SharedPreferences
) : GemPreferencesStore {

    override fun get(key: String): String? {
        return sharedPreferences.getString(key, null)
    }

    override fun set(key: String, value: String) {
        sharedPreferences.edit(commit = true) { putString(key, value) }
    }

    override fun remove(key: String) {
        sharedPreferences.edit(commit = true) { remove(key) }
    }

    override fun clear() {
        sharedPreferences.edit(commit = true) { clear() }
    }

    fun observe(key: String): Flow<String?> = callbackFlow {
        trySend(get(key))
        val listener = SharedPreferences.OnSharedPreferenceChangeListener { _, changed ->
            if (changed == null || changed == key) trySend(get(key))
        }
        sharedPreferences.registerOnSharedPreferenceChangeListener(listener)
        awaitClose { sharedPreferences.unregisterOnSharedPreferenceChangeListener(listener) }
    }.distinctUntilChanged()
}
