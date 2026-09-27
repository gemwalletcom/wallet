package com.gemwallet.android.features.settings.viewmodels.lock.models

data class LockUIState(val initialAuth: AuthState = AuthState.Required, val authState: AuthState? = null, val authPromptRequest: Int = 0, val hasUnlockedApp: Boolean = false) {
    val isUnlocked: Boolean get() = initialAuth == AuthState.Success
}
