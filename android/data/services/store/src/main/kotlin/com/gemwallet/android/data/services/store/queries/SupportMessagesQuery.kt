package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.data.services.store.database.SupportMessagesDao
import com.gemwallet.android.data.services.store.database.entities.toModel
import com.wallet.core.primitives.SupportMessage
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class SupportMessagesQuery @Inject constructor(private val supportMessagesDao: SupportMessagesDao) {

    operator fun invoke(): Flow<List<SupportMessage>> = supportMessagesDao.getMessages().map { records -> records.map { it.toModel() } }
}
