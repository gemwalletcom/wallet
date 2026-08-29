package com.gemwallet.android.features.bridge.viewmodels.model

import com.gemwallet.android.application.wallet_connect.WalletConnectVerifyContext
import uniffi.gemstone.GemWalletConnectService
import uniffi.gemstone.WalletConnectionVerificationStatus
import javax.inject.Inject

class WalletConnectOriginVerifier @Inject constructor(
    private val walletConnectService: GemWalletConnectService,
) {

    fun verify(
        metadataUrl: String?,
        verifyContext: WalletConnectVerifyContext,
    ): OriginVerification {
        val status = walletConnectService.validateOrigin(
            metadataUrl = metadataUrl ?: "",
            origin = verifyContext.origin,
            validation = verifyContext.map(),
        )
        return OriginVerification(status)
    }
}

data class OriginVerification(
    val status: WalletConnectionVerificationStatus,
) {
    val isScam: Boolean
        get() = when (status) {
            WalletConnectionVerificationStatus.INVALID,
            WalletConnectionVerificationStatus.MALICIOUS -> true
            WalletConnectionVerificationStatus.UNKNOWN,
            WalletConnectionVerificationStatus.VERIFIED -> false
        }
}
