package com.gemwallet.android.data.services.gemapi

import uniffi.gemstone.AlienTarget
import java.security.MessageDigest
import java.util.Base64

internal const val NATIVE_PROVIDER_CACHE_HEADER = "x-gem-cache-ttl"
private const val CACHE_KEY_SEPARATOR: Byte = 0

internal fun AlienTarget.nativeCacheKey(): String? {
    if (headers?.containsKey(NATIVE_PROVIDER_CACHE_HEADER) != true) {
        return null
    }
    val digest = MessageDigest.getInstance("SHA-256")
    digest.update(method.name.toByteArray())
    digest.update(CACHE_KEY_SEPARATOR)
    digest.update(url.toByteArray())
    body?.let {
        digest.update(CACHE_KEY_SEPARATOR)
        digest.update(it)
    }
    return Base64.getEncoder().encodeToString(digest.digest())
}
