package com.gemwallet.android.ext

import java.net.URI

fun String.getShortUrl(): String? {
    val value = trim()
    if (value.isEmpty()) {
        return null
    }
    val host = try {
        URI(value).host
    } catch (_: Exception) {
        null
    }
    val fallbackHost = value
        .substringAfter("://", value)
        .substringBefore("/")
        .substringBefore("?")
        .substringBefore("#")

    return (host ?: fallbackHost)
        .takeIf { it.isNotBlank() }
        ?.removePrefix("www.")
}
