package com.gemwallet.android.data.services.nativeprovider

import com.gemwallet.android.ext.isNetworkUnavailable
import com.gemwallet.android.ext.toGatewayNetworkMessage
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import uniffi.gemstone.AlienException
import uniffi.gemstone.AlienProvider
import uniffi.gemstone.AlienResponse
import uniffi.gemstone.AlienTarget
import uniffi.gemstone.Chain
import uniffi.gemstone.GatewayException
import uniffi.gemstone.GemNodeServiceInterface
import java.io.IOException

class NativeProvider(
    private val nodeService: GemNodeServiceInterface,
    private val httpClient: OkHttpClient = OkHttpClient(),
) : AlienProvider {
    override fun getEndpoint(chain: Chain): String = nodeService.nodeUrl(chain)

    override suspend fun request(target: AlienTarget): AlienResponse = withContext(Dispatchers.IO) {
        val requestBuilder = Request.Builder()
            .url(target.url)
            .method(target.method.name, target.body?.toRequestBody())
        target.headers?.forEach { (key, value) -> requestBuilder.addHeader(key, value) }
        try {
            httpClient.newCall(requestBuilder.build()).execute().use { response ->
                AlienResponse(response.code.toUShort(), response.body.bytes())
            }
        } catch (err: IOException) {
            if (err.isNetworkUnavailable()) {
                throw AlienException.Offline()
            }
            throw AlienException.RequestException(err.toGatewayNetworkMessage())
        } catch (err: CancellationException) {
            throw err
        } catch (_: Exception) {
            AlienResponse(500.toUShort(), byteArrayOf())
        }
    }
}
