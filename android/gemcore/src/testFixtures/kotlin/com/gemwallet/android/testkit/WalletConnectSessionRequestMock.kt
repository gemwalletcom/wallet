package com.gemwallet.android.testkit

import com.gemwallet.android.application.wallet_connect.WalletConnectJsonRpcRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectSessionRequest

fun mockWalletConnectSessionRequest(
    id: Long = 1,
    topic: String = "topic",
) = WalletConnectSessionRequest(
    topic = topic,
    chainId = "eip155:1",
    request = WalletConnectJsonRpcRequest(id = id, method = "personal_sign", params = "[]"),
)
