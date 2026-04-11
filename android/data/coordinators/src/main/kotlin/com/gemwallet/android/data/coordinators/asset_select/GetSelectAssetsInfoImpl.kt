package com.gemwallet.android.data.coordinators.asset_select

import com.gemwallet.android.application.asset_select.coordinators.GetSelectAssetsInfo
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.model.AssetInfo
import kotlinx.coroutines.flow.Flow

class GetSelectAssetsInfoImpl(
    private val assetsRepository: AssetsRepository,
) : GetSelectAssetsInfo {
    override fun invoke(): Flow<List<AssetInfo>> = assetsRepository.getAssetsInfo()
}
