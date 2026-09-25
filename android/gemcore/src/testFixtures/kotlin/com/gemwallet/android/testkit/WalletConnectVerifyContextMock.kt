package com.gemwallet.android.testkit

import com.gemwallet.android.application.wallet_connect.WalletConnectValidation
import com.gemwallet.android.application.wallet_connect.WalletConnectVerifyContext

fun mockWalletConnectVerifyContext(origin: String = "", validation: WalletConnectValidation = WalletConnectValidation.Valid, isScam: Boolean = false) = WalletConnectVerifyContext(
    origin = origin,
    validation = validation,
    isScam = isScam,
)
