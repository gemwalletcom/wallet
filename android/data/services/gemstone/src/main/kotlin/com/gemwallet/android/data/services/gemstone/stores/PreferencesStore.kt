package com.gemwallet.android.data.services.gemstone.stores

import android.content.SharedPreferences
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.distinctUntilChanged
import uniffi.gemstone.GemPreferencesStore
import uniffi.gemstone.GemServiceException

class GemstonePreferencesStore(private val sharedPreferences: SharedPreferences) : GemPreferencesStore {

    override fun get(key: String): String? = sharedPreferences.getString(key, null)

    override fun set(key: String, value: String) {
        val previous = mapOf(key to get(key))
        commit(previous) { putString(key, value) }
    }

    override fun remove(key: String) {
        val previous = mapOf(key to get(key))
        commit(previous) { remove(key) }
    }

    override fun clear() {
        val previous = sharedPreferences.all.mapNotNull { (key, value) -> (value as? String)?.let { key to it } }.toMap()
        commit(previous) { clear() }
    }

    fun observe(key: String): Flow<String?> = callbackFlow {
        val listener = SharedPreferences.OnSharedPreferenceChangeListener { _, changed ->
            if (changed == null || changed == key) trySend(get(key))
        }
        sharedPreferences.registerOnSharedPreferenceChangeListener(listener)
        trySend(get(key))
        awaitClose { sharedPreferences.unregisterOnSharedPreferenceChangeListener(listener) }
    }.distinctUntilChanged()

    private fun commit(previous: Map<String, String?>, change: SharedPreferences.Editor.() -> Unit) {
        val editor = sharedPreferences.edit()
        editor.change()
        if (editor.commit()) {
            return
        }
        val restore = sharedPreferences.edit()
        previous.forEach { (key, value) -> if (value == null) restore.remove(key) else restore.putString(key, value) }
        restore.commit()
        throw GemServiceException.Store("preference write failed")
    }
}
