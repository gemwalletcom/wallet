package com.gemwallet.android.application.wallet_connect.cases

import com.gemwallet.android.application.wallet_connect.WalletConnectAuthObject
import com.gemwallet.android.application.wallet_connect.WalletConnectAuthPayloadParams
import com.gemwallet.android.application.wallet_connect.WalletConnectAuthenticationRequest
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
