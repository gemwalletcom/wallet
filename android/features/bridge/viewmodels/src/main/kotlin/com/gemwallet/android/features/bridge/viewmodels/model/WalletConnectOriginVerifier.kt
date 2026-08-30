package com.gemwallet.android.features.bridge.viewmodels.model

import com.gemwallet.android.application.wallet_connect.WalletConnectVerifyContext
import uniffi.gemstone.GemWalletConnectService
import javax.inject.Inject

class WalletConnectOriginVerifier @Inject constructor(
    private val walletConnectService: GemWalletConnectService,
) {

    fun isRejected(
        metadataUrl: String?,
        verifyContext: WalletConnectVerifyContext,
    ): Boolean = walletConnectService.isOriginRejected(
        metadataUrl = metadataUrl ?: "",
        origin = verifyContext.origin,
        validation = verifyContext.map(),
    )
}
