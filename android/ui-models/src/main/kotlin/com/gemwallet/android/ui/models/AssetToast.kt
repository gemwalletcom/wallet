package com.gemwallet.android.ui.models

import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.receiveAsFlow

sealed interface AssetToast {
    data class Pin(val name: String, val pinned: Boolean) : AssetToast
    data object AddedToWallet : AssetToast
}

interface AssetToastEmitter {
    val toastEvents: Flow<AssetToast>

    fun emitToast(toast: AssetToast)
}

class AssetToastEmitterImpl : AssetToastEmitter {
    private val channel = Channel<AssetToast>(Channel.BUFFERED)

    override val toastEvents: Flow<AssetToast> = channel.receiveAsFlow()

    override fun emitToast(toast: AssetToast) {
        channel.trySend(toast)
    }
}
