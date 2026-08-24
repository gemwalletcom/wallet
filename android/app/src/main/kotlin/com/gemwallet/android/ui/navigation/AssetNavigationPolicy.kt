package com.gemwallet.android.ui.navigation

import com.gemwallet.android.ext.hasNativeAsset
import com.gemwallet.android.ext.type
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetSubtype
import com.wallet.core.primitives.Wallet

fun interface AssetNavigationPolicy {
    fun canOpen(assetId: AssetId): Boolean
}

class WalletAssetNavigationPolicy(wallet: Wallet?) : AssetNavigationPolicy {
    private val supportedChains = wallet?.accounts?.map { it.chain }?.toSet() ?: emptySet()

    override fun canOpen(assetId: AssetId): Boolean {
        if (assetId.chain !in supportedChains) return false
        return when (assetId.type()) {
            AssetSubtype.NATIVE -> assetId.chain.hasNativeAsset()
            AssetSubtype.TOKEN -> true
        }
    }
}
