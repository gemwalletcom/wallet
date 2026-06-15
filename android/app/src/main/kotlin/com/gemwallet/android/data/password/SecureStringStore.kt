package com.gemwallet.android.data.password

internal interface SecureStringStore {
    fun contains(key: String): Boolean

    fun getString(key: String): String?

    fun putString(key: String, value: String)

    fun removeString(key: String): Boolean
}

internal fun SecureStringStore.getOrMigrate(legacyStore: SecureStringStore, key: String): String? {
    val value = getString(key) ?: legacyStore.getString(key)?.also {
        putString(key, it)
    } ?: return null
    legacyStore.removeString(key)
    return value
}
