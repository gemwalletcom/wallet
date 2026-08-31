package com.gemwallet.android.features.bridge.viewmodels.model

import com.gemwallet.android.application.wallet_connect.WalletConnectValidation
import com.gemwallet.android.application.wallet_connect.WalletConnectVerifyContext
import uniffi.gemstone.WalletConnectionVerificationStatus

fun WalletConnectVerifyContext.map(): WalletConnectionVerificationStatus {
    if (isScam == true) return WalletConnectionVerificationStatus.MALICIOUS

    return when (this.validation) {
        WalletConnectValidation.Valid -> WalletConnectionVerificationStatus.VERIFIED
        WalletConnectValidation.Invalid -> WalletConnectionVerificationStatus.INVALID
        WalletConnectValidation.Unknown -> WalletConnectionVerificationStatus.UNKNOWN
    }
}
