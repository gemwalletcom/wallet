package com.gemwallet.android.application.nft.cases

import com.gemwallet.android.domains.nft.NftAssetDetailsData
import com.wallet.core.primitives.NFTAssetId
import kotlinx.coroutines.flow.Flow

interface GetNftAssetDetails {
    operator fun invoke(assetId: NFTAssetId): Flow<NftAssetDetailsData?>
}
