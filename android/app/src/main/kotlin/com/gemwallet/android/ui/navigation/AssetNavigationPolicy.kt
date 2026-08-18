package com.gemwallet.android.ui.navigation

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Wallet

fun interface AssetNavigationPolicy {
    fun canOpen(assetId: AssetId): Boolean
}

class WalletAssetNavigationPolicy(wallet: Wallet?) : AssetNavigationPolicy {
    private val supportedChains = wallet?.accounts?.map { it.chain }?.toSet() ?: emptySet()

    override fun canOpen(assetId: AssetId): Boolean = assetId.chain in supportedChains
}
