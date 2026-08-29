package com.gemwallet.android.data.services.nativeprovider

import java.util.concurrent.ConcurrentHashMap

internal class MemoryCache {
    private val entries = ConcurrentHashMap<String, ByteArray>()

    fun get(key: String): ByteArray? {
        return entries[key]?.copyOf()
    }

    fun set(key: String, value: ByteArray) {
        entries[key] = value.copyOf()
    }
}
