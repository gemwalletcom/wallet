package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.services.store.database.SupportMessagesDao
import com.gemwallet.android.data.services.store.database.entities.toRecord
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.SupportAgent
import com.wallet.core.primitives.SupportMessageStatus
import com.wallet.core.primitives.SupportTypingStatus
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import uniffi.gemstone.GemSupportStore
import uniffi.gemstone.SupportMessage as GemSupportMessage
import uniffi.gemstone.SupportTyping as GemSupportTyping

class GemstoneSupportStore(private val supportMessagesDao: SupportMessagesDao) : GemSupportStore {

    private val agent = MutableStateFlow<SupportAgent?>(null)
    val typingAgent: StateFlow<SupportAgent?> = agent.asStateFlow()

    override fun updateTyping(typing: GemSupportTyping) {
        agent.value = typing.toPrimitives().let {
            when (it.status) {
                SupportTypingStatus.On -> it.agent
                SupportTypingStatus.Off -> null
            }
        }
    }

    override fun clearTyping() {
        agent.value = null
    }

    override suspend fun saveMessages(messages: List<GemSupportMessage>) {
        supportMessagesDao.addMessages(messages.map { it.toPrimitives().toRecord() })
    }

    override suspend fun saveMessage(id: String, message: GemSupportMessage) {
        supportMessagesDao.replace(id, message.toPrimitives().toRecord())
    }

    override suspend fun failPendingMessages(exceptIds: List<String>) {
        supportMessagesDao.failPending(
            sending = SupportMessageStatus.Sending.string,
            failed = SupportMessageStatus.Failed.string,
            exceptIds = exceptIds,
        )
    }
}
