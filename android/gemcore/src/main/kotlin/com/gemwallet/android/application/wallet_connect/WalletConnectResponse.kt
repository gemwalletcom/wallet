package com.gemwallet.android.application.wallet_connect

import uniffi.gemstone.GemWalletConnectResponse
import uniffi.gemstone.GemWalletConnectRpcError
import uniffi.gemstone.WalletConnectResponseType

fun GemWalletConnectResponse.toJsonRpcResponse(): WalletConnectJsonRpcResponse = when (this) {
    is GemWalletConnectResponse.Response -> WalletConnectJsonRpcResponse.Result(value.payload())
    GemWalletConnectResponse.Null -> WalletConnectJsonRpcResponse.Result("null")
    is GemWalletConnectResponse.Error -> error.toJsonRpcResponse()
}

fun GemWalletConnectRpcError.toJsonRpcResponse(): WalletConnectJsonRpcResponse.Error =
    WalletConnectJsonRpcResponse.Error(code = code, message = message)

private fun WalletConnectResponseType.payload(): String = when (this) {
    is WalletConnectResponseType.Object -> json
    is WalletConnectResponseType.String -> value
}
