package com.gemwallet.android.application

interface WalletPasswordProtection {
    fun authenticationRequired(): Boolean
    suspend fun setAuthenticationRequired(required: Boolean)
}
