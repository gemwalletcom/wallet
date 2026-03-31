package com.gemwallet.android.testkit

import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Chain

fun mockAccount(
    chain: Chain = Chain.Bitcoin,
    address: String = "wallet-address",
    derivationPath: String = "m/44'/0'/0'/0/0",
    extendedPublicKey: String? = null,
) = Account(
    chain = chain,
    address = address,
    derivationPath = derivationPath,
    extendedPublicKey = extendedPublicKey,
)
