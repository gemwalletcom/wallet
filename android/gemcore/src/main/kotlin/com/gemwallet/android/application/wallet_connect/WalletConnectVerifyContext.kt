package com.gemwallet.android.application.wallet_connect

import uniffi.gemstone.WalletConnectionVerificationStatus

fun WalletConnectVerifyContext.map(): WalletConnectionVerificationStatus {
    if (isScam == true) return WalletConnectionVerificationStatus.MALICIOUS

    return when (this.validation) {
        WalletConnectValidation.Valid -> WalletConnectionVerificationStatus.VERIFIED
        WalletConnectValidation.Invalid -> WalletConnectionVerificationStatus.INVALID
        WalletConnectValidation.Unknown -> WalletConnectionVerificationStatus.UNKNOWN
    }
}
