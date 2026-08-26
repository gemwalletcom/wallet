package com.gemwallet.android.data.repositories.support

import com.gemwallet.android.data.service.store.database.SupportMessagesDao
import com.gemwallet.android.data.service.store.database.entities.toModel
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.wallet.core.primitives.SupportAgent
import com.wallet.core.primitives.SupportMessage
import com.wallet.core.primitives.SupportMessageImage
import com.wallet.core.primitives.SupportMessageInput
import com.wallet.core.primitives.SupportMessageSender
import com.wallet.core.primitives.SupportMessageStatus
import com.wallet.core.primitives.SupportTyping
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import java.util.UUID
import uniffi.gemstone.GemSupportService
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson

class SupportChatRepository(
    private val supportService: GemSupportService,
    private val supportMessagesDao: SupportMessagesDao,
    private val supportTypingState: SupportTypingState,
) {

    val typing: StateFlow<SupportAgent?> = supportTypingState.agent

    fun clearTyping() = supportTypingState.clear()

    fun updateTyping(typing: SupportTyping) = supportTypingState.update(typing)

    fun getMessages(): Flow<List<SupportMessage>> =
        supportMessagesDao.getMessages().map { records -> records.map { it.toModel() } }

    suspend fun syncMessages(fromTimestamp: Long) {
        addMessages(supportService.getMessages(fromTimestamp.toULong()).map { it.decodeJson<SupportMessage>() })
    }

    suspend fun addMessages(messages: List<SupportMessage>) {
        supportMessagesDao.addMessages(messages.map { it.toRecord() })
    }

    suspend fun failPendingMessages() {
        supportMessagesDao.failPending(
            sending = SupportMessageStatus.Sending.string,
            failed = SupportMessageStatus.Failed.string,
        )
    }

    suspend fun sendText(content: String) {
        val message = pendingMessage()
        deliver(message.copy(content = content)) {
            supportService.sendMessage(SupportMessageInput(content = content).toJson()).decodeJson<SupportMessage>()
        }
    }

    suspend fun sendImage(attachment: ImageAttachment) {
        val id = UUID.randomUUID().toString()
        val message = pendingMessage(id = id).copy(
            images = listOf(
                SupportMessageImage(
                    id = id,
                    url = "",
                    fileName = attachment.fileName,
                    fileSize = attachment.data.size.toLong(),
                ),
            ),
        )
        deliver(message) {
            supportService.sendImage(
                image = attachment.data,
                fileName = attachment.fileName,
                mimeType = attachment.mimeType,
            ).decodeJson<SupportMessage>()
        }
    }

    suspend fun retryMessage(message: SupportMessage) {
        deliver(message.copy(status = SupportMessageStatus.Sending)) {
            supportService.sendMessage(SupportMessageInput(content = message.content).toJson()).decodeJson<SupportMessage>()
        }
    }

    private suspend fun deliver(message: SupportMessage, send: suspend () -> SupportMessage) {
        supportMessagesDao.addMessages(listOf(message.toRecord()))
        try {
            supportMessagesDao.replace(message.id, send().toRecord())
        } catch (_: Throwable) {
            supportMessagesDao.addMessages(listOf(message.copy(status = SupportMessageStatus.Failed).toRecord()))
        }
    }

    private fun pendingMessage(id: String = UUID.randomUUID().toString()): SupportMessage = SupportMessage(
        id = id,
        content = "",
        sender = SupportMessageSender.User,
        status = SupportMessageStatus.Sending,
        createdAt = System.currentTimeMillis(),
        images = emptyList(),
    )
}
