package com.gemwallet.android.serializer

import com.gemwallet.android.ext.urlDecode
import com.gemwallet.android.ext.urlEncode
import java.util.Base64

inline fun <reified T> T.packRoutePayload(): String? = runCatching {
    Base64.getEncoder().encodeToString(jsonEncoder.encodeToString(this).toByteArray()).urlEncode()
}.getOrNull()

inline fun <reified T> unpackRoutePayload(input: String): T? = runCatching {
    jsonEncoder.decodeFromString<T>(String(Base64.getDecoder().decode(input.urlDecode())))
}.getOrNull()
