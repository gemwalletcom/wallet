package com.gemwallet.android.serializer

import kotlinx.serialization.decodeFromString
import kotlinx.serialization.encodeToString

inline fun <reified T> T.encodeArg(): String = jsonEncoder.encodeToString(this)

inline fun <reified T> String?.decodeArg(): T? =
    this?.let { runCatching { jsonEncoder.decodeFromString<T>(it) }.getOrNull() }
