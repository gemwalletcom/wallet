package com.gemwallet.android.data.repositories.stream

import android.util.Log
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.support.SupportChatRepository
import com.gemwallet.android.serializer.StreamEventSerializer
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.StreamEvent
import com.wallet.core.primitives.SupportMessageSender
import com.wallet.core.primitives.SupportStreamEvent
import kotlinx.serialization.json.Json
import uniffi.gemstone.GemStreamService

class StreamEventHandler(
    private val streamService: GemStreamService,
    private val sessionRepository: SessionRepository,
    private val supportChatRepository: SupportChatRepository,
    private val json: Json = Json { ignoreUnknownKeys = true },
) {
    suspend fun handle(text: String) {
        try {
            streamService.handle(text, sessionRepository.getCurrentCurrency().toJson())
            val event = json.decodeFromString(StreamEventSerializer, text)
            if (event is StreamEvent.Support) {
                handleSupport(event.data)
            }
        } catch (err: Throwable) {
            Log.e(TAG, "Event handler error", err)
        }
    }

    private fun handleSupport(event: SupportStreamEvent) {
        when (event) {
            is SupportStreamEvent.Message -> when (event.data.sender) {
                is SupportMessageSender.User -> { }
                is SupportMessageSender.Agent -> supportChatRepository.clearTyping()
            }
            is SupportStreamEvent.Typing -> supportChatRepository.updateTyping(event.data)
        }
    }

    companion object {
        private const val TAG = "StreamEventHandler"
    }
}
