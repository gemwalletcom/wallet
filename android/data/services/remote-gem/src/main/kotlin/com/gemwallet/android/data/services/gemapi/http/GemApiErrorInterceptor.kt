package com.gemwallet.android.data.services.gemapi.http

import com.gemwallet.android.data.services.gemapi.GemApiException
import com.gemwallet.android.data.services.gemapi.models.ResponseError
import okhttp3.Interceptor
import okhttp3.Response

class GemApiErrorInterceptor : Interceptor {

    override fun intercept(chain: Interceptor.Chain): Response {
        val response = chain.proceed(chain.request())
        val responseBody = response.peekBody(64L * 1024)

        ResponseError.parseOrNull(responseBody.string())?.let { error ->
            throw GemApiException(error.error.message)
        }

        return response
    }
}
