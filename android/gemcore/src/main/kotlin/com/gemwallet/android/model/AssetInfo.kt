package com.gemwallet.android.model

import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetAssociation
import com.wallet.core.primitives.AssetMetaData
import com.wallet.core.primitives.WalletId

data class AssetInfo(
    val owner: Account?,
    val asset: Asset,
    val balance: AssetBalance = AssetBalance(asset),
    val walletId: WalletId?,
    val price: AssetPriceInfo? = null,
    val metadata: AssetMetaData? = null,
    val associations: List<AssetAssociation> = emptyList(),
) {
    fun id() = asset.id
}
