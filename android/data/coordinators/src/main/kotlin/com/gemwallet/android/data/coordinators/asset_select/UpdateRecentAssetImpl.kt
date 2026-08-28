package com.gemwallet.android.data.coordinators.asset_select

import com.gemwallet.android.application.asset_select.cases.UpdateRecentAsset
import com.gemwallet.android.data.repositories.assets.RecentAssetsService
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.model.RecentType
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first

class UpdateRecentAssetImpl(
    private val sessionRepository: SessionRepository,
    private val recentAssetsService: RecentAssetsService,
) : UpdateRecentAsset {
    override suspend fun invoke(assetId: AssetId, type: RecentType) {
        val wallet = sessionRepository.session().filterNotNull().first().wallet
        recentAssetsService.addRecentActivity(assetId, wallet.id.id, type)
    }
}
