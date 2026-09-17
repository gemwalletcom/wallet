package com.gemwallet.android.ui.models

import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.receiveAsFlow

data class ToastMessage(
    val title: String,
    val image: Int,
)

interface ToastEmitter {
    val toastEvents: Flow<ToastMessage>

    fun emitToast(toast: ToastMessage)
}

class ToastEmitterImpl : ToastEmitter {
    private val channel = Channel<ToastMessage>(Channel.BUFFERED)

    override val toastEvents: Flow<ToastMessage> = channel.receiveAsFlow()

    override fun emitToast(toast: ToastMessage) {
        channel.trySend(toast)
    }
}
