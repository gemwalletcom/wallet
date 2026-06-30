package com.gemwallet.android.serializer

import kotlinx.serialization.decodeFromString
import kotlinx.serialization.encodeToString

inline fun <reified T> T.toJson(): String = jsonEncoder.encodeToString(this)

inline fun <reified T> String?.fromJson(): T? =
    this?.let { runCatching { jsonEncoder.decodeFromString<T>(it) }.getOrNull() }
