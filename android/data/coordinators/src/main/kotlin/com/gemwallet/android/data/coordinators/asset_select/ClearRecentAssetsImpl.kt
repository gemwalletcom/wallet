package com.gemwallet.android.data.coordinators.asset_select

import com.gemwallet.android.application.asset_select.cases.ClearRecentAssets
import com.gemwallet.android.data.repositories.assets.RecentAssetsService
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.model.RecentType
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first

class ClearRecentAssetsImpl(
    private val sessionRepository: SessionRepository,
    private val recentAssetsService: RecentAssetsService,
) : ClearRecentAssets {
    override suspend fun invoke(types: List<RecentType>) {
        val wallet = sessionRepository.session().filterNotNull().first().wallet
        recentAssetsService.clearRecentAssets(wallet.id, types)
    }
}
