package com.gemwallet.android.model

import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetAssociation
import com.wallet.core.primitives.AssetMetaData
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId

data class AssetData(
    val asset: Asset,
    val balance: AssetBalance = AssetBalance(asset),
    val account: Account,
    val walletId: WalletId,
    val price: AssetPriceInfo? = null,
    val metadata: AssetMetaData,
    val associations: List<AssetAssociation> = emptyList(),
) {
    fun toAssetInfo(): AssetInfo = AssetInfo(
        owner = account,
        asset = asset,
        balance = balance,
        walletId = walletId,
        price = price,
        metadata = metadata,
        associations = associations,
    )

    companion object {
        fun from(assetInfo: AssetInfo, wallet: Wallet, account: Account) = AssetData(
            asset = assetInfo.asset,
            account = account,
            walletId = wallet.id,
            balance = assetInfo.balance,
            price = assetInfo.price,
            metadata = assetInfo.metadata,
            associations = assetInfo.associations,
        )
    }
}
