package com.gemwallet.android.application.assets.coordinators

import com.wallet.core.primitives.AssetId

interface EnableAsset {
    suspend operator fun invoke(walletId: String, assetId: AssetId)

    suspend operator fun invoke(walletId: String, assetIds: List<AssetId>)
}
