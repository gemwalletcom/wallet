package com.gemwallet.android.data.coordinators.asset_select

import com.gemwallet.android.application.asset_select.cases.UpdateRecentAsset
import com.gemwallet.android.data.services.gemstone.assets.RecentAssetsService
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.model.RecentType
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first

class UpdateRecentAssetImpl(
    private val getSession: GetSession,
    private val recentAssetsService: RecentAssetsService,
) : UpdateRecentAsset {
    override suspend fun invoke(assetId: AssetId, type: RecentType) {
        val wallet = getSession().filterNotNull().first().wallet
        recentAssetsService.addRecentActivity(assetId, wallet.id.id, type)
    }
}
