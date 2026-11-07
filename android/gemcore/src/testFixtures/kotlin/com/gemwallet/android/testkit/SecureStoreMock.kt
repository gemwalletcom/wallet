package com.gemwallet.android.testkit

import uniffi.gemstone.GemSecureStore

class SecureStoreMock(private val values: MutableMap<String, String> = mutableMapOf()) : GemSecureStore {
    override fun get(key: String): String? = values[key]

    override fun set(key: String, value: String) {
        values[key] = value
    }

    override fun remove(key: String) {
        values.remove(key)
    }
}
