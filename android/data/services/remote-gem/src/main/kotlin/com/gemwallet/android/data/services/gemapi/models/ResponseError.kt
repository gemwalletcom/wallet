package com.gemwallet.android.data.services.gemapi.models

import com.gemwallet.android.serializer.jsonEncoder
import kotlinx.serialization.Serializable

@Serializable
data class ResponseError(val error: ErrorDescription) {
    @Serializable
    data class ErrorDescription(val message: String)

    companion object {
        fun parseOrNull(body: String): ResponseError? =
            runCatching { jsonEncoder.decodeFromString<ResponseError>(body) }.getOrNull()
    }
}
