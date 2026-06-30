package com.gemwallet.android.data.services.gemapi.models

import com.gemwallet.android.serializer.fromJson
import kotlinx.serialization.Serializable

@Serializable
data class ResponseError(val error: ErrorDescription) {
    @Serializable
    data class ErrorDescription(val message: String)

    companion object {
        fun parseOrNull(body: String): ResponseError? =
            body.fromJson<ResponseError>()
    }
}
