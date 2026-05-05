package com.gemwallet.android.testkit

import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.AssetData
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetMetaData
import com.wallet.core.primitives.Wallet

fun mockAssetData(
    asset: Asset = mockAsset(),
    wallet: Wallet = mockWallet(accounts = listOf(mockAccount(chain = asset.id.chain))),
    account: Account = mockAccount(chain = asset.id.chain),
    balance: AssetBalance = AssetBalance.create(asset),
    metadata: AssetMetaData? = null,
) = AssetData.from(
    assetInfo = mockAssetInfo(
        asset = asset,
        balance = balance,
        metadata = metadata,
    ),
    wallet = wallet,
    account = account,
)
