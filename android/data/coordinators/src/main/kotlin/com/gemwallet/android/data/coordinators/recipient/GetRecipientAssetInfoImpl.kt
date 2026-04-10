package com.gemwallet.android.data.coordinators.recipient

import com.gemwallet.android.application.recipient.coordinators.GetRecipientAssetInfo
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.Flow

class GetRecipientAssetInfoImpl(
    private val assetsRepository: AssetsRepository,
) : GetRecipientAssetInfo {
    override fun getAssetInfo(assetId: AssetId): Flow<AssetInfo?> = assetsRepository.getAssetInfo(assetId)
}
