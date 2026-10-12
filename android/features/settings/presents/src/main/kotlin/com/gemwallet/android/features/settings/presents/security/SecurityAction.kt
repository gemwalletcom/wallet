package com.gemwallet.android.features.settings.presents.security

internal sealed interface SecurityAction {
    data object Cancel : SecurityAction
    data class SetAuthRequired(val required: Boolean) : SecurityAction
    data object HideBalances : SecurityAction
    data class SetLockInterval(val minutes: Int) : SecurityAction
}
