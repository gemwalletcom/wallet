package com.gemwallet.android.application.nft.cases

import com.wallet.core.primitives.NFTAssetId

interface RefreshNftAsset {
    suspend operator fun invoke(assetId: NFTAssetId)
}
