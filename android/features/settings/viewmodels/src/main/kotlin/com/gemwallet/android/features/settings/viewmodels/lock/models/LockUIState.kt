package com.gemwallet.android.features.settings.viewmodels.lock.models

import androidx.compose.runtime.Immutable

@Immutable
data class LockUIState(val initialAuth: AuthState = AuthState.Required, val authState: AuthState? = null, val authPromptRequest: Int = 0) {
    val isUnlocked: Boolean get() = initialAuth == AuthState.Success
}
