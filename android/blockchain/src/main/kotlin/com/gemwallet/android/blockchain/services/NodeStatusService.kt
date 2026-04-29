package com.gemwallet.android.blockchain.services

import com.gemwallet.android.model.NodeStatus
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemGateway
import java.net.HttpURLConnection
import java.net.URL

class NodeStatusService(
    private val gateway: GemGateway,
) {

    suspend fun getNodeStatus(chain: Chain, url: String): NodeStatus? {
        return try {
            val result = gateway.getNodeStatus(chain.string, url)
            NodeStatus(
                url = url,
                chainId = result.chainId,
                blockNumber = result.latestBlockNumber,
                inSync = true,
                latency = result.latencyMs,
            )
        } catch (_: Throwable) {
            null
        }
    }

    suspend fun getEndpointLatency(url: String): ULong? {
        return try {
            measureEndpointLatency(url)
        } catch (error: CancellationException) {
            throw error
        } catch (_: Throwable) {
            null
        }
    }

    private suspend fun measureEndpointLatency(url: String): ULong = withContext(Dispatchers.IO) {
        val start = System.nanoTime()
        val connection = (URL(url).openConnection() as HttpURLConnection).apply {
            requestMethod = "GET"
            connectTimeout = 10_000
            readTimeout = 10_000
            useCaches = false
        }
        try {
            connection.responseCode
            runCatching { connection.errorStream?.close() }
            runCatching { connection.inputStream?.close() }
            ((System.nanoTime() - start) / 1_000_000).toULong()
        } finally {
            connection.disconnect()
        }
    }
}
