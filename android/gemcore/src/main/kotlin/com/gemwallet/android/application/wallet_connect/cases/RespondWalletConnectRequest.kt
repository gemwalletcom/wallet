package com.gemwallet.android.application.wallet_connect.cases

import uniffi.gemstone.GemWalletConnectFailure
import uniffi.gemstone.GemWalletConnectSessionRequest

interface RespondWalletConnectRequest {
    fun respondRequest(request: GemWalletConnectSessionRequest, onFailure: (GemWalletConnectFailure) -> Unit, onFinish: () -> Unit)
    fun rejectRequest(request: GemWalletConnectSessionRequest, onFinish: () -> Unit)
}
