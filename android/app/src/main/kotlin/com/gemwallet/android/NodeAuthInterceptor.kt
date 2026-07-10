package com.gemwallet.android

import com.gemwallet.android.cases.nodes.GemNodeRegion
import okhttp3.HttpUrl.Companion.toHttpUrl
import okhttp3.Interceptor
import okhttp3.Response
import uniffi.gemstone.GemPreferences

class NodeAuthInterceptor(
    private val preferences: GemPreferences,
    private val currentTimeSeconds: () -> ULong = { (System.currentTimeMillis() / 1_000).toULong() },
) : Interceptor {
    private val nodesDomain = GemNodeRegion.US.baseUrl.toHttpUrl().host

    override fun intercept(chain: Interceptor.Chain): Response {
        val request = chain.request()
        val host = request.url.host
        if (host != nodesDomain && !host.endsWith(".$nodesDomain")) return chain.proceed(request)
        val token = preferences.authToken()
        if (token != null && token.expiresAt > currentTimeSeconds()) {
            return chain.proceed(
                request.newBuilder()
                    .header("Authorization", "Bearer ${token.token}")
                    .build()
            )
        }
        return chain.proceed(request)
    }
}
