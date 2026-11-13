package com.gemwallet.android.data.services.nativeprovider

import com.gemwallet.android.ext.isNetworkUnavailable
import com.gemwallet.android.ext.toGatewayNetworkMessage
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.awaitCancellation
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody
import okhttp3.RequestBody.Companion.toRequestBody
import uniffi.gemstone.AlienException
import uniffi.gemstone.AlienHttpMethod
import uniffi.gemstone.AlienProvider
import uniffi.gemstone.AlienResponse
import uniffi.gemstone.AlienTarget
import uniffi.gemstone.alienMethodToString
import java.io.IOException

class NativeProvider(private val httpClient: OkHttpClient = OkHttpClient()) : AlienProvider {
    override suspend fun request(target: AlienTarget): AlienResponse = coroutineScope {
        val requestBuilder = Request.Builder()
            .url(target.url)
            .method(alienMethodToString(target.method), target.requestBody())
        target.headers?.forEach { (key, value) -> requestBuilder.addHeader(key, value) }
        val call = httpClient.newCall(requestBuilder.build())
        val cancelOnExit = launch(start = CoroutineStart.UNDISPATCHED) {
            try {
                awaitCancellation()
            } finally {
                call.cancel()
            }
        }
        try {
            withContext(Dispatchers.IO) {
                call.execute().use { response -> AlienResponse(response.code.toUShort(), response.body.bytes()) }
            }
        } catch (error: IOException) {
            if (error.isNetworkUnavailable()) {
                throw AlienException.Offline()
            }
            throw AlienException.RequestException(error.toGatewayNetworkMessage())
        } finally {
            cancelOnExit.cancel()
        }
    }
}

private fun AlienTarget.requestBody(): RequestBody? = body?.toRequestBody() ?: when (method) {
    AlienHttpMethod.POST, AlienHttpMethod.PUT, AlienHttpMethod.PATCH -> ByteArray(0).toRequestBody()
    AlienHttpMethod.GET, AlienHttpMethod.DELETE, AlienHttpMethod.HEAD, AlienHttpMethod.OPTIONS -> null
}
