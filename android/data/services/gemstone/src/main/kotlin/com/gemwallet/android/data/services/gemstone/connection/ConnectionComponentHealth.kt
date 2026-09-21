package com.gemwallet.android.data.services.gemstone.connection

import com.wallet.core.primitives.ConnectionComponent
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow

class ConnectionComponentHealth(override val component: ConnectionComponent) : ConnectionComponentMonitor {
    private val health = MutableSharedFlow<Boolean>(replay = 1, extraBufferCapacity = 1)

    fun report(isHealthy: Boolean) {
        health.tryEmit(isHealthy)
    }

    override fun healthFlow(): Flow<Boolean> = health.asSharedFlow()
}
