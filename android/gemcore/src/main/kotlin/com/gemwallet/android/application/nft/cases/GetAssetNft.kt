package com.gemwallet.android.application.nft.cases

import com.wallet.core.primitives.NFTAssetId
import com.wallet.core.primitives.NFTData
import kotlinx.coroutines.flow.Flow

interface GetAssetNft {
    fun getAssetNft(assetId: NFTAssetId): Flow<NFTData>
}
