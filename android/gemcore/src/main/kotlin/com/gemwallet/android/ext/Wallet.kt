package com.gemwallet.android.ext

import com.gemwallet.android.domains.asset.assetConfig
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletType

fun Wallet.getAccount(chain: Chain): Account? {
    return accounts.firstOrNull { it.chain == chain }
}

fun Wallet.getAccount(assetId: AssetId): Account? = getAccount(assetId.chain)

val WalletType.isViewOnly: Boolean get() = this == WalletType.View

val HypercoreUSDC: Asset = requireNotNull(assetConfig.defaultAsset(Chain.HyperCore.string, AssetType.PERPETUAL.toGem())) {
    "Missing perpetual default asset for HyperCore"
}.toPrimitives()
