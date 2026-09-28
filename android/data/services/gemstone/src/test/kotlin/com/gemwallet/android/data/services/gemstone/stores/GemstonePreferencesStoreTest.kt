package com.gemwallet.android.data.services.gemstone.stores

import android.content.SharedPreferences
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Test
import uniffi.gemstone.GemServiceException

class GemstonePreferencesStoreTest {

    @Test
    fun aWriteWhileTheObserverRegistersIsNotMissed() = runTest {
        val preferences = MemorySharedPreferences(mutableMapOf("wallet" to "a"))
        preferences.onRegister = { preferences.write("wallet", "b") }

        assertEquals("b", GemstonePreferencesStore(preferences).observe("wallet").first())
    }

    @Test
    fun aFailedCommitThrowsAndKeepsTheStoredValue() {
        val preferences = MemorySharedPreferences(mutableMapOf("currency" to "USD"))
        preferences.failCommits = 1
        val store = GemstonePreferencesStore(preferences)

        assertThrows(GemServiceException.Store::class.java) { store.set("currency", "EUR") }
        assertEquals("USD", store.get("currency"))

        preferences.failCommits = 1
        assertThrows(GemServiceException.Store::class.java) { store.remove("currency") }
        assertEquals("USD", store.get("currency"))

        preferences.failCommits = 1
        assertThrows(GemServiceException.Store::class.java) { store.clear() }
        assertEquals("USD", store.get("currency"))
    }
}

private class MemorySharedPreferences(private val values: MutableMap<String, String>) : SharedPreferences {
    var onRegister: (() -> Unit)? = null
    var failCommits = 0
    private val listeners = mutableListOf<SharedPreferences.OnSharedPreferenceChangeListener>()

    fun write(key: String, value: String) {
        values[key] = value
        listeners.forEach { it.onSharedPreferenceChanged(this, key) }
    }

    override fun getAll(): Map<String, *> = values.toMap()
    override fun getString(key: String?, defValue: String?): String? = values[key] ?: defValue
    override fun getStringSet(key: String?, defValues: MutableSet<String>?): MutableSet<String>? = defValues
    override fun getInt(key: String?, defValue: Int): Int = defValue
    override fun getLong(key: String?, defValue: Long): Long = defValue
    override fun getFloat(key: String?, defValue: Float): Float = defValue
    override fun getBoolean(key: String?, defValue: Boolean): Boolean = defValue
    override fun contains(key: String?): Boolean = values.containsKey(key)
    override fun edit(): SharedPreferences.Editor = Editor()

    override fun registerOnSharedPreferenceChangeListener(listener: SharedPreferences.OnSharedPreferenceChangeListener) {
        onRegister?.invoke()
        listeners.add(listener)
    }

    override fun unregisterOnSharedPreferenceChangeListener(listener: SharedPreferences.OnSharedPreferenceChangeListener) {
        listeners.remove(listener)
    }

    private inner class Editor : SharedPreferences.Editor {
        private val changes = mutableListOf<(MutableMap<String, String>) -> Unit>()

        override fun putString(key: String, value: String?): SharedPreferences.Editor = also { changes.add { it[key] = value.orEmpty() } }
        override fun putStringSet(key: String?, values: MutableSet<String>?): SharedPreferences.Editor = this
        override fun putInt(key: String?, value: Int): SharedPreferences.Editor = this
        override fun putLong(key: String?, value: Long): SharedPreferences.Editor = this
        override fun putFloat(key: String?, value: Float): SharedPreferences.Editor = this
        override fun putBoolean(key: String?, value: Boolean): SharedPreferences.Editor = this
        override fun remove(key: String): SharedPreferences.Editor = also { changes.add { it.remove(key) } }
        override fun clear(): SharedPreferences.Editor = also { changes.add { it.clear() } }

        override fun commit(): Boolean {
            changes.forEach { it(values) }
            if (failCommits > 0) {
                failCommits -= 1
                return false
            }
            return true
        }

        override fun apply() {
            changes.forEach { it(values) }
        }
    }
}
