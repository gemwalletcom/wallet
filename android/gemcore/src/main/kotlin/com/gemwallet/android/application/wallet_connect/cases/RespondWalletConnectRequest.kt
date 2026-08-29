package com.gemwallet.android.application.wallet_connect.cases

import com.gemwallet.android.application.wallet_connect.WalletConnectJsonRpcResponse

interface RespondWalletConnectRequest {
    fun respond(topic: String, id: Long, response: WalletConnectJsonRpcResponse, onSuccess: () -> Unit, onError: (String) -> Unit)
}
