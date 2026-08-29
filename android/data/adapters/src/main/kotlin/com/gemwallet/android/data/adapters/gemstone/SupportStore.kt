package com.gemwallet.android.data.adapters.gemstone

import com.gemwallet.android.data.service.store.database.SupportMessagesDao
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.SupportAgent
import com.gemwallet.android.data.service.store.database.entities.toModel
import com.wallet.core.primitives.SupportMessage
import com.wallet.core.primitives.SupportMessageStatus
import com.wallet.core.primitives.SupportTyping
import com.wallet.core.primitives.SupportTypingStatus
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import uniffi.gemstone.GemSupportStore

class GemstoneSupportStore(
    private val supportMessagesDao: SupportMessagesDao,
) : GemSupportStore {

    private val agent = MutableStateFlow<SupportAgent?>(null)
    val typingAgent: StateFlow<SupportAgent?> = agent.asStateFlow()

    override fun updateTyping(typing: String) {
        agent.value = typing.decodeJson<SupportTyping>().let {
            when (it.status) {
                SupportTypingStatus.On -> it.agent
                SupportTypingStatus.Off -> null
            }
        }
    }

    override fun clearTyping() {
        agent.value = null
    }

    override suspend fun saveMessages(messages: List<String>) {
        supportMessagesDao.addMessages(messages.map { it.decodeJson<SupportMessage>().toRecord() })
    }

    override suspend fun saveMessage(id: String, message: String) {
        supportMessagesDao.replace(id, message.decodeJson<SupportMessage>().toRecord())
    }

    fun observeMessages(): Flow<List<SupportMessage>> =
        supportMessagesDao.getMessages().map { records -> records.map { it.toModel() } }

    suspend fun failPendingMessages() {
        supportMessagesDao.failPending(
            sending = SupportMessageStatus.Sending.string,
            failed = SupportMessageStatus.Failed.string,
        )
    }
}
