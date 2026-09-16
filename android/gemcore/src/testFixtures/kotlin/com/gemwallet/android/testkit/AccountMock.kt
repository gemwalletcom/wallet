package com.gemwallet.android.testkit

import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Chain

fun mockAccount(
    chain: Chain = Chain.Bitcoin,
    address: String = "wallet-address",
) = Account(
    chain = chain,
    address = address,
    derivationPath = "m/44'/0'/0'/0/0",
    extendedPublicKey = null,
)
