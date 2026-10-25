package com.gemwallet.android.features.settings.viewmodels.lock.models

import androidx.compose.runtime.Immutable
import uniffi.gemstone.GemErrorText

@Immutable
data class LockUIState(
    val initialAuth: AuthState = AuthState.Required,
    val authState: AuthState? = null,
    val authPromptRequest: Int = 0,
    val hasUnlockedApp: Boolean = false,
    val isLocalDataReset: Boolean = false,
    val resetError: GemErrorText? = null,
) {
    val isUnlocked: Boolean get() = initialAuth == AuthState.Success
}
