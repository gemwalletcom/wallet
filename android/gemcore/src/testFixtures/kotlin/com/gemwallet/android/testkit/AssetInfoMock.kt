package com.gemwallet.android.testkit

import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetAssociation
import com.wallet.core.primitives.AssetMetaData
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletType

fun mockAssetInfo(
    asset: Asset = mockAsset(),
    owner: Account? = mockAccount(asset.id.chain),
    balance: AssetBalance = AssetBalance.create(asset),
    walletId: WalletId? = mockWalletId(),
    walletType: WalletType = WalletType.View,
    walletName: String = "Wallet",
    metadata: AssetMetaData? = null,
    associations: List<AssetAssociation> = emptyList(),
) = AssetInfo(
    owner = owner,
    asset = asset,
    balance = balance,
    walletId = walletId,
    walletType = walletType,
    walletName = walletName,
    metadata = metadata,
    associations = associations,
)
