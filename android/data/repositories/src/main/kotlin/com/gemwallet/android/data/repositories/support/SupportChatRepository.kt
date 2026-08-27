package com.gemwallet.android.data.repositories.support

import com.gemwallet.android.data.service.store.database.SupportMessagesDao
import com.gemwallet.android.data.service.store.database.entities.toModel
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.SupportAgent
import com.wallet.core.primitives.SupportMessage
import com.wallet.core.primitives.SupportMessageStatus
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import uniffi.gemstone.GemSupportService

class SupportChatRepository(
    private val supportService: GemSupportService,
    private val supportMessagesDao: SupportMessagesDao,
    private val supportTypingState: SupportTypingState,
) {
    val typing: StateFlow<SupportAgent?> = supportTypingState.agent

    fun clearTyping() = supportTypingState.clear()



    fun getMessages(): Flow<List<SupportMessage>> =
        supportMessagesDao.getMessages().map { records -> records.map { it.toModel() } }

    suspend fun syncMessages(fromTimestamp: Long) = supportService.syncMessages(fromTimestamp.toULong())

    suspend fun addMessages(messages: List<SupportMessage>) {
        supportMessagesDao.addMessages(messages.map { it.toRecord() })
    }

    suspend fun failPendingMessages() {
        supportMessagesDao.failPending(
            sending = SupportMessageStatus.Sending.string,
            failed = SupportMessageStatus.Failed.string,
        )
    }

    suspend fun sendText(content: String) = runCatching { supportService.sendText(content) }.getOrDefault(Unit)

    suspend fun sendImage(attachment: ImageAttachment) = runCatching {
        supportService.sendImage(attachment.data, attachment.fileName, attachment.mimeType)
    }.getOrDefault(Unit)

    suspend fun retryMessage(message: SupportMessage) = runCatching { supportService.retryMessage(message.toJson()) }.getOrDefault(Unit)
}
