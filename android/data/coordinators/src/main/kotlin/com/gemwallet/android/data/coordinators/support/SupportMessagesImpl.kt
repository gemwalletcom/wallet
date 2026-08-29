package com.gemwallet.android.data.coordinators.support

import com.gemwallet.android.application.support.cases.ClearSupportTyping
import com.gemwallet.android.application.support.cases.FailPendingSupportMessages
import com.gemwallet.android.application.support.cases.GetSupportMessages
import com.gemwallet.android.application.support.cases.GetSupportTyping
import com.gemwallet.android.data.repositories.gemstone.GemstoneSupportStore
import com.gemwallet.android.data.service.store.database.SupportMessagesDao
import com.gemwallet.android.data.service.store.database.entities.toModel
import com.wallet.core.primitives.SupportAgent
import com.wallet.core.primitives.SupportMessage
import com.wallet.core.primitives.SupportMessageStatus
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map

class GetSupportMessagesImpl(
    private val supportMessagesDao: SupportMessagesDao,
) : GetSupportMessages {

    override fun invoke(): Flow<List<SupportMessage>> =
        supportMessagesDao.getMessages().map { records -> records.map { it.toModel() } }
}

class FailPendingSupportMessagesImpl(
    private val supportMessagesDao: SupportMessagesDao,
) : FailPendingSupportMessages {

    override suspend fun invoke() {
        supportMessagesDao.failPending(
            sending = SupportMessageStatus.Sending.string,
            failed = SupportMessageStatus.Failed.string,
        )
    }
}

class SupportTypingCoordinator(
    private val supportStore: GemstoneSupportStore,
) : GetSupportTyping, ClearSupportTyping {

    override fun typingAgent(): StateFlow<SupportAgent?> = supportStore.typingAgent

    override fun clearTyping() = supportStore.clearTyping()
}
