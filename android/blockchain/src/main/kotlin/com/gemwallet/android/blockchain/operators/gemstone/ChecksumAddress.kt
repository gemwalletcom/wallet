package com.gemwallet.android.blockchain.operators.gemstone

import com.wallet.core.primitives.Chain

fun Chain.checksumAddress(address: String): String =
    uniffi.gemstone.checksumAddress(address = address, chain = string)
