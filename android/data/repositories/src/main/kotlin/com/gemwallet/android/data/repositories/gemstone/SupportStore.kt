package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.service.store.database.SupportMessagesDao
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.SupportMessage
import uniffi.gemstone.GemSupportStore

class GemstoneSupportStore(
    private val supportMessagesDao: SupportMessagesDao,
) : GemSupportStore {

    override suspend fun saveMessages(messages: List<String>) {
        supportMessagesDao.addMessages(messages.map { it.decodeJson<SupportMessage>().toRecord() })
    }

    override suspend fun replaceMessage(id: String, message: String) {
        supportMessagesDao.replace(id, message.decodeJson<SupportMessage>().toRecord())
    }
}
