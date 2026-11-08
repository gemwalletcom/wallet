package com.gemwallet.android.data.coordinators.wallet_connect

import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import uniffi.gemstone.GemWalletConnectEvent
import uniffi.gemstone.GemWalletConnectListener

class WalletConnectListener : GemWalletConnectListener {
    private val walletEvents = MutableSharedFlow<GemWalletConnectEvent>(
        replay = 1,
        extraBufferCapacity = 64,
        onBufferOverflow = BufferOverflow.DROP_OLDEST,
    )
    val events: SharedFlow<GemWalletConnectEvent> = walletEvents.asSharedFlow()

    override fun onEvent(event: GemWalletConnectEvent) {
        walletEvents.tryEmit(event)
    }
}
