package com.gemwallet.android.data.coordinators.wallet_connect

import com.gemwallet.android.data.services.gemstone.stream.WebSocketConnectable
import com.gemwallet.android.data.services.gemstone.stream.WebSocketEvent
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.onCompletion
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.withTimeoutOrNull
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemWalletConnectTransport
import kotlin.time.Duration.Companion.seconds

class WalletConnectRelay(private val connection: WebSocketConnectable) : GemWalletConnectTransport {
    private val isConnected = MutableStateFlow(false)

    fun connect(): Flow<WebSocketEvent> = connection.connect()
        .onEach { event ->
            when (event) {
                WebSocketEvent.Connected -> isConnected.value = true
                WebSocketEvent.Disconnected -> isConnected.value = false
                is WebSocketEvent.Message -> Unit
            }
        }
        .onCompletion { isConnected.value = false }

    override suspend fun send(message: String) {
        if (connection.send(message)) {
            return
        }
        withTimeoutOrNull(OPEN_TIMEOUT) { isConnected.first { it } } ?: throw GemServiceException.Offline()
        if (!connection.send(message)) {
            throw GemServiceException.Offline()
        }
    }

    private companion object {
        val OPEN_TIMEOUT = 10.seconds
    }
}
