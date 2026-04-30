package com.gemwallet.android.application.nft.coordinators

import com.gemwallet.android.domains.nft.NftAssetDetailsData
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.Flow

interface GetNftAssetDetails {
    operator fun invoke(assetId: AssetId): Flow<NftAssetDetailsData?>
}
