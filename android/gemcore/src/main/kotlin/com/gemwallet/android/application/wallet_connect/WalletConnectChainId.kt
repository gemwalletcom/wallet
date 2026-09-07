package com.gemwallet.android.application.wallet_connect

import com.gemwallet.android.ext.requireChain
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemChainServiceInterface

fun Chain.Companion.fromWalletConnectChainId(chainService: GemChainServiceInterface, walletConnectChainId: String?): Chain? {
    val chainId = walletConnectChainId ?: return null
    return chainService.chainFromCaip2(chainId)?.requireChain()
}
