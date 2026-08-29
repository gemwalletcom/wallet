package com.gemwallet.android.application.bridge.cases

import com.gemwallet.android.application.bridge.WalletConnectJsonRpcResponse

interface RespondWalletConnectRequest {
    fun respond(topic: String, id: Long, response: WalletConnectJsonRpcResponse, onSuccess: () -> Unit, onError: (String) -> Unit)
}
