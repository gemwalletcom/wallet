package com.gemwallet.android.testkit

import com.gemwallet.android.application.wallet_connect.WalletConnectValidation
import com.gemwallet.android.application.wallet_connect.WalletConnectVerifyContext

fun mockWalletConnectVerifyContext() = WalletConnectVerifyContext(
    origin = mockApplicationMetadata().url,
    validation = WalletConnectValidation.Valid,
    isScam = false,
)
