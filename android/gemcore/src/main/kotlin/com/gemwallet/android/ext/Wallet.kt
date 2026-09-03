package com.gemwallet.android.ext

import com.gemwallet.android.domains.asset.defaultAssets
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

val HypercoreUSDC: Asset = Chain.HyperCore.defaultAssets
    .first { it.type == AssetType.PERPETUAL }
