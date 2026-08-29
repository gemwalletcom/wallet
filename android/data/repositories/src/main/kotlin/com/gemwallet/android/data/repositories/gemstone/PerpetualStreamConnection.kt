package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.repositories.stream.WebSocketConnectable
import uniffi.gemstone.GemPerpetualStreamConnection

class GemstonePerpetualStreamConnection(
    private val connection: WebSocketConnectable,
) : GemPerpetualStreamConnection {

    override suspend fun send(message: String) {
        check(connection.send(message)) { "Perpetual stream connection is closed" }
    }
}
