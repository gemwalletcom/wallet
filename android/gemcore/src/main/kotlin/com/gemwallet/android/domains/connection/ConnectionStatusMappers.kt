package com.gemwallet.android.domains.connection

import com.wallet.core.primitives.ConnectionComponent
import com.wallet.core.primitives.ConnectionStatus
import uniffi.gemstone.GemConnectionComponent
import uniffi.gemstone.GemConnectionStatus
import uniffi.gemstone.connectionStatus

fun List<ConnectionComponent>.toConnectionStatus(): ConnectionStatus =
    connectionStatus(map { it.toGem() }).toPrimitives()

private fun ConnectionComponent.toGem(): GemConnectionComponent = when (this) {
    ConnectionComponent.Internet -> GemConnectionComponent.INTERNET
    ConnectionComponent.Api -> GemConnectionComponent.API
    ConnectionComponent.Nodes -> GemConnectionComponent.NODES
    ConnectionComponent.Stream -> GemConnectionComponent.STREAM
}

private fun GemConnectionStatus.toPrimitives(): ConnectionStatus = when (this) {
    GemConnectionStatus.ONLINE -> ConnectionStatus.Online
    GemConnectionStatus.NO_INTERNET -> ConnectionStatus.NoInternet
    GemConnectionStatus.NO_SERVICE -> ConnectionStatus.NoService
}
