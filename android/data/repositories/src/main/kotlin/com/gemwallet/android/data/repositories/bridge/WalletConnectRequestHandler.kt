package com.gemwallet.android.data.repositories.bridge

import uniffi.gemstone.GemWalletConnectRequest
import uniffi.gemstone.GemWalletConnectResponse
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemWalletConnectServiceInterface
import uniffi.gemstone.WalletConnectResponseType

class WalletConnectRequestHandler(
    private val service: GemWalletConnectServiceInterface,
) {
    suspend fun handle(request: WalletConnectSessionRequest, domain: String): WalletConnectJsonRpcResponse {
        val chainId = request.chainId ?: return rejected()
        return try {
            service.handleRequest(
                GemWalletConnectRequest(
                    topic = request.topic,
                    method = request.request.method,
                    params = request.request.params,
                    chainId = chainId,
                    domain = domain,
                ),
            ).toJsonRpcResponse()
        } catch (err: GemServiceException.Cancelled) {
            rejected()
        }
    }

    private fun rejected() = WalletConnectJsonRpcResponse.Error(code = 4001, message = "User rejected the request")

}

fun GemWalletConnectResponse.toJsonRpcResponse(): WalletConnectJsonRpcResponse = when (this) {
    is GemWalletConnectResponse.Response -> WalletConnectJsonRpcResponse.Result(value.payload())
    GemWalletConnectResponse.Null -> WalletConnectJsonRpcResponse.Result("null")
    GemWalletConnectResponse.MethodNotFound -> WalletConnectJsonRpcResponse.Error(code = -32601, message = "Method not found")
}

fun WalletConnectResponseType.payload(): String = when (this) {
    is WalletConnectResponseType.Object -> json
    is WalletConnectResponseType.String -> value
}
