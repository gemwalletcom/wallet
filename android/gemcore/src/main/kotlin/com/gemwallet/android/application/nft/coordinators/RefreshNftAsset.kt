package com.gemwallet.android.application.nft.coordinators

import com.wallet.core.primitives.AssetId

interface RefreshNftAsset {
    suspend operator fun invoke(assetId: AssetId)
}
