package com.gemwallet.android.data.repositories.support

import com.wallet.core.primitives.SupportAgent
import com.wallet.core.primitives.SupportTyping
import com.wallet.core.primitives.SupportTypingStatus
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

class SupportTypingState {

    private val state = MutableStateFlow<SupportAgent?>(null)
    val agent: StateFlow<SupportAgent?> = state.asStateFlow()

    fun update(typing: SupportTyping) {
        state.value = when (typing.status) {
            SupportTypingStatus.On -> typing.agent
            SupportTypingStatus.Off -> null
        }
    }

    fun clear() {
        state.value = null
    }
}
