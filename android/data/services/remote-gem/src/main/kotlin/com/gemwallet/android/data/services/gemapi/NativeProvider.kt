package com.gemwallet.android.data.services.gemapi

import com.gemwallet.android.cases.nodes.GetCurrentNodeCase
import com.gemwallet.android.cases.nodes.GetNodesCase
import com.gemwallet.android.cases.nodes.SetCurrentNodeCase
import com.gemwallet.android.data.services.gemapi.http.getNodeUrl
import com.gemwallet.android.ext.toChain
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
import java.io.IOException

class NativeProvider(
    private val getNodesCase: GetNodesCase,
    private val getCurrentNodeCase: GetCurrentNodeCase,
    private val setCurrentNodeCase: SetCurrentNodeCase,
    private val httpClient: OkHttpClient = OkHttpClient(),
    private val config: NativeProviderConfig,
) : AlienProvider {
    private val cache = MemoryCache()

    override fun getEndpoint(chain: Chain): String {
        return chain.toChain()?.getNodeUrl(getNodesCase, getCurrentNodeCase, setCurrentNodeCase)
            ?: throw GatewayException.PlatformException("Can't found node url for chain: $chain")
    }

    override suspend fun request(target: AlienTarget): AlienResponse = withContext(Dispatchers.IO) {
        val cacheKey = target.nativeCacheKey()
        if (cacheKey != null) {
            cache.get(cacheKey)?.let {
                return@withContext AlienResponse(200.toUShort(), it)
            }
        }

        val requestBuilder = Request.Builder()
            .url(target.url)
            .method(target.method.name, target.body?.toRequestBody())
        target.headers?.forEach { (key, value) ->
            if (key != NATIVE_PROVIDER_CACHE_HEADER) {
                requestBuilder.addHeader(key, value)
            }
        }
        try {
            httpClient.newCall(requestBuilder.build()).execute().use { response ->
                val data = response.body.bytes()
                val status = response.code.toUShort()
                if (cacheKey != null) {
                    cache.set(cacheKey, data)
                }
                AlienResponse(status, data)
            }
        } catch (err: IOException) {
            throw AlienException.RequestException(err.toGatewayNetworkMessage(config.networkOfflineMessage))
        } catch (err: CancellationException) {
            throw err
        } catch (_: Exception) {
            AlienResponse(500.toUShort(), byteArrayOf())
        }
    }
}
