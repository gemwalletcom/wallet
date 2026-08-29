package com.gemwallet.android.data.coordinators.support

import com.gemwallet.android.application.support.cases.ClearSupportTyping
import com.gemwallet.android.application.support.cases.FailPendingSupportMessages
import com.gemwallet.android.application.support.cases.GetSupportMessages
import com.gemwallet.android.application.support.cases.GetSupportTyping
import com.gemwallet.android.data.adapters.gemstone.GemstoneSupportStore
import com.wallet.core.primitives.SupportAgent
import com.wallet.core.primitives.SupportMessage
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.StateFlow

class GetSupportMessagesImpl(
    private val supportStore: GemstoneSupportStore,
) : GetSupportMessages {

    override fun invoke(): Flow<List<SupportMessage>> = supportStore.observeMessages()
}

class FailPendingSupportMessagesImpl(
    private val supportStore: GemstoneSupportStore,
) : FailPendingSupportMessages {

    override suspend fun invoke() = supportStore.failPendingMessages()
}

class SupportTypingCoordinator(
    private val supportStore: GemstoneSupportStore,
) : GetSupportTyping, ClearSupportTyping {

    override fun typingAgent(): StateFlow<SupportAgent?> = supportStore.typingAgent

    override fun clearTyping() = supportStore.clearTyping()
}
