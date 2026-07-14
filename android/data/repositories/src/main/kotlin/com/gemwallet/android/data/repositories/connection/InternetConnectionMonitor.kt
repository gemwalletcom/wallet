package com.gemwallet.android.data.repositories.connection

import android.content.Context
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import com.wallet.core.primitives.ConnectionComponent
import com.wallet.core.primitives.ConnectionComponentHealth
import com.wallet.core.primitives.ConnectionComponentMetadata
import com.wallet.core.primitives.InternetConnectionMetadata
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

    override fun healthFlow(): Flow<ConnectionComponentHealth> = callbackFlow {
        val connectivityManager = context.getSystemService(ConnectivityManager::class.java)
        val callback = object : ConnectivityManager.NetworkCallback() {
            override fun onCapabilitiesChanged(network: Network, capabilities: NetworkCapabilities) {
                trySend(connectivityManager.health(capabilities))
            }

            override fun onLost(network: Network) {
                trySend(ConnectionComponentHealth(isHealthy = false, metadata = null))
            }
        }
        trySend(connectivityManager.currentHealth())
        connectivityManager.registerDefaultNetworkCallback(callback)
        awaitClose { connectivityManager.unregisterNetworkCallback(callback) }
    }
        .mapLatest { health ->
            if (!health.isHealthy) {
                delay(OFFLINE_DEBOUNCE_MILLISECONDS)
            }
            health
        }
        .distinctUntilChanged()

    private fun ConnectivityManager.currentHealth(): ConnectionComponentHealth {
        val capabilities = getNetworkCapabilities(activeNetwork)
            ?: return ConnectionComponentHealth(isHealthy = false, metadata = null)
        return health(capabilities)
    }

    private fun ConnectivityManager.health(capabilities: NetworkCapabilities): ConnectionComponentHealth {
        val isHealthy = capabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET)
            && capabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_VALIDATED)
        return ConnectionComponentHealth(
            isHealthy = isHealthy,
            metadata = ConnectionComponentMetadata.Internet(
                InternetConnectionMetadata(
                    isLowData = !capabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_NOT_METERED)
                        || restrictBackgroundStatus == ConnectivityManager.RESTRICT_BACKGROUND_STATUS_ENABLED,
                    isVpn = capabilities.hasTransport(NetworkCapabilities.TRANSPORT_VPN),
                )
            ),
        )
    }

    private companion object {
        const val OFFLINE_DEBOUNCE_MILLISECONDS = 500L
    }
}
