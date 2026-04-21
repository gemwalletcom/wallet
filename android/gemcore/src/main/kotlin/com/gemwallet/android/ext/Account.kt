package com.gemwallet.android.ext

import com.wallet.core.primitives.Account
import uniffi.gemstone.GemAccount

fun Account.toGemAccount(): GemAccount = GemAccount(
    chain = chain.string,
    address = address,
    derivationPath = derivationPath,
    publicKey = publicKey,
    extendedPublicKey = extendedPublicKey,
)
