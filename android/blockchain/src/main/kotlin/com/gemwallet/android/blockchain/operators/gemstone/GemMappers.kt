package com.gemwallet.android.blockchain.operators.gemstone

import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemKeystoreAccount

internal fun GemKeystoreAccount.toAccount(): Account = Account(
    chain = chain.toPrimitiveChain(),
    address = address,
    derivationPath = derivationPath,
    extendedPublicKey = publicKey.orEmpty(),
)

internal fun String.toPrimitiveChain(): Chain =
    Chain.entries.firstOrNull { it.string == this }
        ?: throw IllegalArgumentException("Unsupported chain: $this")
