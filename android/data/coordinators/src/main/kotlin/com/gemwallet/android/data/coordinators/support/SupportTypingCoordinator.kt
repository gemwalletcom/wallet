package com.gemwallet.android.data.coordinators.support

import com.gemwallet.android.application.support.cases.ClearSupportTyping
import com.gemwallet.android.application.support.cases.GetSupportTyping
import com.gemwallet.android.data.services.gemstone.stores.GemstoneSupportStore
import com.wallet.core.primitives.SupportAgent
import kotlinx.coroutines.flow.StateFlow

class SupportTypingCoordinator(private val supportStore: GemstoneSupportStore) :
    GetSupportTyping,
    ClearSupportTyping {

    override fun typingAgent(): StateFlow<SupportAgent?> = supportStore.typingAgent

    override fun clearTyping() = supportStore.clearTyping()
}
