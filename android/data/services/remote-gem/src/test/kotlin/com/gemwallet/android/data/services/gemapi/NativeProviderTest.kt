package com.gemwallet.android.data.services.gemapi

import com.gemwallet.android.cases.nodes.GetCurrentNodeCase
import com.gemwallet.android.cases.nodes.GetNodesCase
import com.gemwallet.android.cases.nodes.SetCurrentNodeCase
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Node
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.runBlocking
import okhttp3.OkHttpClient
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.AlienException
import uniffi.gemstone.AlienHttpMethod
import uniffi.gemstone.AlienTarget
import java.io.EOFException
import java.io.IOException
import java.net.UnknownHostException

class NativeProviderTest {

    @Test
    fun requestMapsKnownOfflineIoErrors() {
        val provider = nativeProvider(
            httpClient = OkHttpClient.Builder()
                .addInterceptor {
                    throw UnknownHostException("api.example.com")
                }
                .build(),
            config = NativeProviderConfig(networkOfflineMessage = "offline"),
        )

        try {
            runBlocking {
                provider.request(
                    AlienTarget(
                        url = "https://gemnodes.com/bitcoin",
                        method = AlienHttpMethod.GET,
                        headers = null,
                        body = null,
                    )
                )
            }
        } catch (err: AlienException.RequestException) {
            assertEquals("offline", err.msg)
            return
        }
        throw AssertionError("Expected offline request exception")
    }

    @Test
    fun requestMapsDroppedStreamToOffline() {
        val provider = nativeProvider(
            httpClient = OkHttpClient.Builder()
                .addInterceptor {
                    throw IOException("unexpected end of stream on https://gemnodes.com/...", EOFException())
                }
                .build(),
            config = NativeProviderConfig(networkOfflineMessage = "offline"),
        )

        try {
            runBlocking {
                provider.request(
                    AlienTarget(
                        url = "https://gemnodes.com/bitcoin",
                        method = AlienHttpMethod.GET,
                        headers = null,
                        body = null,
                    )
                )
            }
        } catch (err: AlienException.RequestException) {
            assertEquals("offline", err.msg)
            return
        }
        throw AssertionError("Expected request exception")
    }

    private fun nativeProvider(
        httpClient: OkHttpClient = OkHttpClient(),
        config: NativeProviderConfig = NativeProviderConfig(networkOfflineMessage = "offline"),
    ): NativeProvider {
        return NativeProvider(
            getNodesCase = object : GetNodesCase {
                override suspend fun getNodes(chain: Chain): Flow<List<Node>> = emptyFlow()
            },
            getCurrentNodeCase = object : GetCurrentNodeCase {
                override fun getCurrentNode(chain: Chain): Node? = null
            },
            setCurrentNodeCase = object : SetCurrentNodeCase {
                override fun setCurrentNode(chain: Chain, node: Node) = Unit
            },
            httpClient = httpClient,
            config = config,
        )
    }
}
