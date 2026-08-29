package com.gemwallet.android.data.services.gemstone.connection

import android.content.Context
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import com.wallet.core.primitives.ConnectionComponent
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.mapLatest

@OptIn(ExperimentalCoroutinesApi::class)
class InternetConnectionMonitor(
    private val context: Context,
) : ConnectionComponentMonitor {

    override val component: ConnectionComponent = ConnectionComponent.Internet

    override fun healthFlow(): Flow<Boolean> = callbackFlow {
        val connectivityManager = context.getSystemService(ConnectivityManager::class.java)
        val callback = object : ConnectivityManager.NetworkCallback() {
            override fun onCapabilitiesChanged(network: Network, capabilities: NetworkCapabilities) {
                trySend(capabilities.isHealthy())
            }

            override fun onLost(network: Network) {
                trySend(false)
            }
        }
        trySend(connectivityManager.currentHealth())
        connectivityManager.registerDefaultNetworkCallback(callback)
        awaitClose { connectivityManager.unregisterNetworkCallback(callback) }
    }
        .mapLatest { isHealthy ->
            if (!isHealthy) {
                delay(OFFLINE_DEBOUNCE_MILLISECONDS)
            }
            isHealthy
        }
        .distinctUntilChanged()

    private fun ConnectivityManager.currentHealth(): Boolean {
        return getNetworkCapabilities(activeNetwork)?.isHealthy() ?: false
    }

    private fun NetworkCapabilities.isHealthy(): Boolean {
        return hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET)
            && hasCapability(NetworkCapabilities.NET_CAPABILITY_VALIDATED)
    }

    private companion object {
        const val OFFLINE_DEBOUNCE_MILLISECONDS = 500L
    }
}
