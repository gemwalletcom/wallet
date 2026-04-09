package com.gemwallet.android.application.nft.coordinators

import com.gemwallet.android.domains.nft.NftAssetDetailsData
import kotlinx.coroutines.flow.Flow

interface GetNftAssetDetails {
    operator fun invoke(assetId: String): Flow<NftAssetDetailsData?>
}
