package com.gemwallet.android.data.service.store.database.entities

import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain

fun mockDbAssetWallet(
    assetId: AssetId = AssetId(Chain.Bitcoin),
    walletId: String = "wallet-1",
    accountAddress: String = "address",
) = DbAssetWallet(
    assetId = assetId.toIdentifier(),
    walletId = walletId,
    accountAddress = accountAddress,
)
