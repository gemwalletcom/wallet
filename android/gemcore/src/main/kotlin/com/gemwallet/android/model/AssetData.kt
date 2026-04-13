package com.gemwallet.android.model

import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetMetaData
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId

// TODO: Move to TypeShare once Balance is typeshared in core.
//  1. Add #[typeshare] to Balance in core/crates/primitives/src/asset_balance.rs
//  2. Create core/crates/primitives/src/asset_data.rs with #[typeshare]
//  3. Remove "asset_data.rs" from Kotlin ignored list in core/bin/generate/src/main.rs:194
//  4. Remove hand-written AssetData.swift from ios/Packages/Primitives/Sources/
//  5. Run generation for both platforms
data class AssetData(
    val asset: Asset,
    val balance: AssetBalance = AssetBalance(asset),
    val account: Account,
    val walletId: WalletId,
    val price: AssetPriceInfo? = null,
    val metadata: AssetMetaData? = null,
) {
    fun toAssetInfo(): AssetInfo = AssetInfo(
        owner = account,
        asset = asset,
        balance = balance,
        walletId = walletId.id,
        price = price,
        metadata = metadata,
    )

    companion object {
        fun from(assetInfo: AssetInfo, wallet: Wallet, account: Account) = AssetData(
            asset = assetInfo.asset,
            account = account,
            walletId = WalletId(wallet.id),
            balance = assetInfo.balance,
            price = assetInfo.price,
            metadata = assetInfo.metadata,
        )
    }
}
