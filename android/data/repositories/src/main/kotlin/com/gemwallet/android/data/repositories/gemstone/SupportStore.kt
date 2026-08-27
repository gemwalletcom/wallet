package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.service.store.database.SupportMessagesDao
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.SupportMessage
import uniffi.gemstone.GemSupportStore
import com.gemwallet.android.data.repositories.support.SupportTypingState
import com.wallet.core.primitives.SupportTyping

class GemstoneSupportStore(
    private val supportMessagesDao: SupportMessagesDao,
    private val supportTypingState: SupportTypingState,
) : GemSupportStore {
    override fun updateTyping(typing: String) = supportTypingState.update(typing.decodeJson<SupportTyping>())

    override fun clearTyping() = supportTypingState.clear()


    override suspend fun saveMessages(messages: List<String>) {
        supportMessagesDao.addMessages(messages.map { it.decodeJson<SupportMessage>().toRecord() })
    }

    override suspend fun replaceMessage(id: String, message: String) {
        supportMessagesDao.replace(id, message.decodeJson<SupportMessage>().toRecord())
    }
}
