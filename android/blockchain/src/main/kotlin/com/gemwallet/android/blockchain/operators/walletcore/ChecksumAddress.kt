package com.gemwallet.android.blockchain.operators.walletcore

import com.gemwallet.android.ext.toEVM
import com.wallet.core.primitives.Chain
import wallet.core.jni.AnyAddress

fun Chain.checksumAddress(address: String): String {
    toEVM() ?: return address
    val coinType = WCChainTypeProxy().invoke(this)
    return runCatching { AnyAddress(address, coinType).description() }.getOrDefault(address)
}
