package com.gemwallet.android.application.wallet_connect.cases

import com.gemwallet.android.application.wallet_connect.WalletConnectJsonRpcResponse
import uniffi.gemstone.GemErrorText

interface RespondWalletConnectRequest {
    fun respond(topic: String, id: Long, response: WalletConnectJsonRpcResponse, onSuccess: () -> Unit, onError: (GemErrorText) -> Unit)
}
