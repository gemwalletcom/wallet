package com.gemwallet.android.application.wallet_connect.cases

import com.gemwallet.android.application.wallet_connect.WalletConnectSessionRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectVerifyContext
import uniffi.gemstone.GemWalletConnectFailure

interface RespondWalletConnectRequest {
    fun respondRequest(request: WalletConnectSessionRequest, verifyContext: WalletConnectVerifyContext, onFailure: (GemWalletConnectFailure) -> Unit, onFinish: () -> Unit)
    fun rejectRequest(request: WalletConnectSessionRequest, onFinish: () -> Unit)
}
