package com.gemwallet.android.application.support.cases

import com.wallet.core.primitives.SupportAgent
import com.wallet.core.primitives.SupportMessage
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.StateFlow

interface GetSupportMessages {
    operator fun invoke(): Flow<List<SupportMessage>>
}

interface FailPendingSupportMessages {
    suspend operator fun invoke()
}

interface GetSupportTyping {
    fun typingAgent(): StateFlow<SupportAgent?>
}

interface ClearSupportTyping {
    fun clearTyping()
}
