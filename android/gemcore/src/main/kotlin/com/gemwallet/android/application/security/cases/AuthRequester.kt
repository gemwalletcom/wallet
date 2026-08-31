package com.gemwallet.android.application.security.cases

import com.gemwallet.android.model.AuthRequest

interface AuthRequester {
    fun requestAuth(auth: AuthRequest, onSuccess: () -> Unit)
}
