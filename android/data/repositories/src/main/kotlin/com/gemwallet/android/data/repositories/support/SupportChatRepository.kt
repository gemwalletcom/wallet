package com.gemwallet.android.data.repositories.support

import com.gemwallet.android.data.service.store.database.SupportMessagesDao
import com.gemwallet.android.data.service.store.database.entities.toModel
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.wallet.core.primitives.SupportMessage
import com.wallet.core.primitives.SupportMessageImage
import com.wallet.core.primitives.SupportMessageInput
import com.wallet.core.primitives.SupportMessageSender
import com.wallet.core.primitives.SupportMessageStatus
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.RequestBody.Companion.toRequestBody
import java.util.UUID

class SupportChatRepository(
    private val gemDeviceApiClient: GemDeviceApiClient,
    private val supportMessagesDao: SupportMessagesDao,
) {

    fun getMessages(): Flow<List<SupportMessage>> =
        supportMessagesDao.getMessages().map { records -> records.map { it.toModel() } }

    suspend fun syncMessages(fromTimestamp: Long) {
        addMessages(gemDeviceApiClient.getSupportMessages(fromTimestamp))
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
            gemDeviceApiClient.sendSupportMessage(SupportMessageInput(content = content))
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
            gemDeviceApiClient.sendSupportImage(
                fileName = attachment.fileName,
                image = attachment.data.toRequestBody(attachment.mimeType.toMediaType()),
            )
        }
    }

    suspend fun retryMessage(message: SupportMessage) {
        deliver(message.copy(status = SupportMessageStatus.Sending)) {
            gemDeviceApiClient.sendSupportMessage(SupportMessageInput(content = message.content))
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
