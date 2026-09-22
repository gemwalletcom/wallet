package com.gemwallet.android.data.services.gemstone.stream

import uniffi.gemstone.GemStreamConnection
import java.time.Duration

class GemstoneStreamConnection(private val connection: WebSocketConnectable) : GemStreamConnection {
    override suspend fun latency(): Duration? = connection.ping()

    override suspend fun isConnected(): Boolean = connection.isConnected

    override suspend fun send(message: String) {
        check(connection.send(message)) { "Stream connection is closed" }
    }
}
