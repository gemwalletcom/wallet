package com.gemwallet.android.data.coordinators.asset_select

import com.gemwallet.android.application.asset_select.cases.ClearRecentAssets
import com.gemwallet.android.data.services.gemstone.assets.RecentAssetsService
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.model.RecentType
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first

class ClearRecentAssetsImpl(
    private val getSession: GetSession,
    private val recentAssetsService: RecentAssetsService,
) : ClearRecentAssets {
    override suspend fun invoke(types: List<RecentType>) {
        val wallet = getSession().filterNotNull().first().wallet
        recentAssetsService.clearRecentAssets(wallet.id, types)
    }
}
