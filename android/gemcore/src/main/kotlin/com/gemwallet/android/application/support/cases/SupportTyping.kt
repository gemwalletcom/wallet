package com.gemwallet.android.application.support.cases

import com.wallet.core.primitives.SupportAgent
import kotlinx.coroutines.flow.StateFlow

interface GetSupportTyping {
    fun typingAgent(): StateFlow<SupportAgent?>
}

interface ClearSupportTyping {
    fun clearTyping()
}
