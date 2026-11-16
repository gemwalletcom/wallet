package com.gemwallet.android.data.services.gemstone.perpetual

import com.gemwallet.android.data.services.gemstone.stream.WebSocketConnectable
import uniffi.gemstone.GemSocketTransport

class GemstonePerpetualStreamConnection(private val connection: WebSocketConnectable) : GemSocketTransport {

    override suspend fun send(message: String) {
        check(connection.send(message)) { "Perpetual stream connection is closed" }
    }
}
