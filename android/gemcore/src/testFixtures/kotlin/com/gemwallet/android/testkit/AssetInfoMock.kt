package com.gemwallet.android.testkit

import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.AssetPriceInfo
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetMetaData
import com.wallet.core.primitives.WalletId

fun mockAssetInfo(
    asset: Asset = mockAsset(),
    owner: Account? = mockAccount(asset.id.chain),
    balance: AssetBalance = AssetBalance.create(asset),
    walletId: WalletId? = mockWalletId(),
    price: AssetPriceInfo? = null,
    metadata: AssetMetaData = mockAssetMetaData(),
) = AssetInfo(
    owner = owner,
    asset = asset,
    balance = balance,
    walletId = walletId,
    price = price,
    metadata = metadata,
)
