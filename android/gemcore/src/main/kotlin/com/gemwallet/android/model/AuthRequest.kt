package com.gemwallet.android.model

enum class AuthRequest {
    Default,
    Confirmation,
}

val AuthRequest.requiresConfirmation: Boolean
    get() = when (this) {
        AuthRequest.Default -> false
        AuthRequest.Confirmation -> true
    }
