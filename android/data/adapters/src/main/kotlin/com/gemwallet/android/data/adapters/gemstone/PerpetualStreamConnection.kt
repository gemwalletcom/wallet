package com.gemwallet.android.data.adapters.gemstone

import com.gemwallet.android.data.adapters.stream.WebSocketConnectable
import uniffi.gemstone.GemPerpetualStreamConnection

class GemstonePerpetualStreamConnection(
    private val connection: WebSocketConnectable,
) : GemPerpetualStreamConnection {

    override suspend fun send(message: String) {
        check(connection.send(message)) { "Perpetual stream connection is closed" }
    }
}
