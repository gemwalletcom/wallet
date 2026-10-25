package com.gemwallet.android.features.settings.viewmodels.lock.models

enum class AuthState {
    Required,
    Success,
    Fail,
    KeyChanged,
    WalletsUnreadable,
    ResetRequested,
    ;

    val requiresPrompt: Boolean get() = this == Required || this == KeyChanged || this == ResetRequested

    val isKeystoreRecovery: Boolean get() = this == KeyChanged || this == WalletsUnreadable || this == ResetRequested
}
