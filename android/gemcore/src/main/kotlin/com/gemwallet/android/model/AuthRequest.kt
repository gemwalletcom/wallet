package com.gemwallet.android.model

enum class AuthRequest {
    Default,
    Confirmation,
    Required,
}

val AuthRequest.requiresConfirmation: Boolean
    get() = when (this) {
        AuthRequest.Default, AuthRequest.Required -> false
        AuthRequest.Confirmation -> true
    }
