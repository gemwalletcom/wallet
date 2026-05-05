package com.gemwallet.android

import java.net.URLDecoder

internal fun String.decodeUriComponent(): String? {
    return runCatching { URLDecoder.decode(this, Charsets.UTF_8.name()) }.getOrNull()
}

internal fun String?.decodeUriQueryParameters(): Map<String, String> {
    val rawQuery = this?.takeIf { it.isNotBlank() } ?: return emptyMap()
    return rawQuery
        .split("&")
        .mapNotNull { pair ->
            if (pair.isBlank()) return@mapNotNull null
            val separator = pair.indexOf("=")
            val rawKey = if (separator < 0) pair else pair.substring(0, separator)
            val rawValue = if (separator < 0) "" else pair.substring(separator + 1)
            val key = rawKey.decodeUriComponent() ?: return@mapNotNull null
            val value = rawValue.decodeUriComponent() ?: return@mapNotNull null
            key to value
        }
        .toMap()
}
