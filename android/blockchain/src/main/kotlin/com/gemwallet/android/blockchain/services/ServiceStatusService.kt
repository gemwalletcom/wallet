package com.gemwallet.android.blockchain.services

import kotlinx.coroutines.CancellationException
import uniffi.gemstone.GemServiceEndpoint
import uniffi.gemstone.GemServiceStatus

class ServiceStatusService(
    private val client: GemServiceStatus,
) {

    fun getEndpoints(): List<GemServiceEndpoint> = client.getEndpoints()

    suspend fun getEndpointLatency(url: String): ULong? {
        return try {
            client.getEndpointLatency(url)
        } catch (error: CancellationException) {
            throw error
        } catch (_: Throwable) {
            null
        }
    }
}
