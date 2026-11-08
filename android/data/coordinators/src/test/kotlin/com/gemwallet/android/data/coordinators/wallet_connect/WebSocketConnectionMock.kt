package com.gemwallet.android.data.coordinators.wallet_connect

import com.gemwallet.android.data.services.gemstone.stream.WebSocketConnectable
import com.gemwallet.android.data.services.gemstone.stream.WebSocketEvent
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.onCompletion
import kotlinx.coroutines.flow.onStart
import java.time.Duration

class WebSocketConnectionMock : WebSocketConnectable {
    val events = MutableSharedFlow<WebSocketEvent>()
    val sent = mutableListOf<String>()
    var isOpen = false
    var connections = 0
        private set
    var isCollected = false
        private set

    override val isConnected: Boolean get() = isOpen

    override val connectionLatency: Duration? = null

    override fun connect(): Flow<WebSocketEvent> = events
        .onStart {
            connections += 1
            isCollected = true
        }
        .onCompletion { isCollected = false }

    override suspend fun send(message: String): Boolean {
        if (isOpen) {
            sent.add(message)
        }
        return isOpen
    }
}
