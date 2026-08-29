package com.gemwallet.android.data.adapters.gemstone

import com.gemwallet.android.data.adapters.stream.WebSocketConnectable
import uniffi.gemstone.GemStreamConnection

class GemstoneStreamConnection(
    private val connection: WebSocketConnectable,
) : GemStreamConnection {
    override suspend fun isConnected(): Boolean = connection.isConnected

    override suspend fun send(message: String) {
        check(connection.send(message)) { "Stream connection is closed" }
    }
}
