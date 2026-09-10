package com.gemwallet.android.ext

import uniffi.gemstone.GemServiceException

fun Throwable.serviceMessage(): String = when (this) {
    is GemServiceException.Api -> msg
    is GemServiceException.Gateway -> msg
    is GemServiceException.Store -> msg
    is GemServiceException.Core -> msg
    is GemServiceException.Platform -> msg
    is GemServiceException.InvalidInput -> msg
    is GemServiceException.NotFound -> msg
    is GemServiceException.Unsupported -> msg
    else -> message ?: toString()
}
