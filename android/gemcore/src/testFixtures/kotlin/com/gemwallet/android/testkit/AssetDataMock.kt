package com.gemwallet.android.testkit

import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.AssetData
import com.gemwallet.android.model.AssetPriceInfo
import com.wallet.core.primitives.AssetMetaData

fun mockAssetData(balance: AssetBalance = AssetBalance.create(mockAsset()), price: AssetPriceInfo? = null, metadata: AssetMetaData = mockAssetMetaData()): AssetData {
    val asset = mockAsset()
    val account = mockAccount(chain = asset.id.chain)
    return AssetData.from(
        assetInfo = mockAssetInfo(
            asset = asset,
            balance = balance,
            price = price,
            metadata = metadata,
        ),
        wallet = mockWallet(accounts = listOf(account)),
        account = account,
    )
}
