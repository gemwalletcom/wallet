package com.gemwallet.android.application.bridge.cases

import com.gemwallet.android.application.bridge.WalletConnectAuthObject
import com.gemwallet.android.application.bridge.WalletConnectAuthPayloadParams
import com.gemwallet.android.application.bridge.WalletConnectAuthenticationRequest
import com.wallet.core.primitives.Wallet

interface ApproveWalletConnectAuthentication {
    fun approveAuthentication(
        request: WalletConnectAuthenticationRequest,
        auths: List<WalletConnectAuthObject>,
        wallet: Wallet,
        onSuccess: () -> Unit,
        onError: (String) -> Unit,
    )

    fun rejectAuthentication(request: WalletConnectAuthenticationRequest, onSuccess: () -> Unit = {}, onError: (String) -> Unit = {})

    fun authPayloadParams(
        payloadParams: WalletConnectAuthPayloadParams,
        supportedChains: List<String>,
        supportedMethods: List<String>,
    ): WalletConnectAuthPayloadParams

    fun authMessage(payloadParams: WalletConnectAuthPayloadParams, issuer: String): String

    fun authObject(payloadParams: WalletConnectAuthPayloadParams, issuer: String, signature: String): WalletConnectAuthObject
}
