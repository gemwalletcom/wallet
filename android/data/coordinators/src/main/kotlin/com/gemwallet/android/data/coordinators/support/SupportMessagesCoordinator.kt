package com.gemwallet.android.data.coordinators.support

import com.gemwallet.android.application.support.cases.SendSupportMessage
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.SupportMessage
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.async
import uniffi.gemstone.GemSupportServiceInterface

class SupportMessagesCoordinator(private val supportService: GemSupportServiceInterface, private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)) : SendSupportMessage {

    override suspend fun sendText(content: String) = deliver { supportService.sendText(content) }

    override suspend fun sendImages(images: List<ByteArray>) = deliver {
        images.map { runCatchingCancellable { supportService.sendImage(it) } }.firstNotNullOfOrNull { it.exceptionOrNull() }?.let { throw it }
    }

    override suspend fun retry(message: SupportMessage) = deliver { supportService.retryMessage(message.toGem()) }

    private suspend fun deliver(send: suspend () -> Unit) = scope.async { send() }.await()
}
