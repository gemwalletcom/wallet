package com.gemwallet.android.data.password

internal interface SecureStringStore {
    fun contains(key: String): Boolean

    fun getString(key: String): String?

    fun putString(key: String, value: String)

    fun removeString(key: String): Boolean
}

internal fun SecureStringStore.getOrMigrate(legacyStore: SecureStringStore, key: String): String? {
    val currentValue = getString(key)
    if (currentValue != null) {
        return currentValue
    }

    val legacyValue = legacyStore.getString(key) ?: return null
    putString(key, legacyValue)
    legacyStore.removeString(key)
    return legacyValue
}
