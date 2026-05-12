package com.gemwallet.android.data.coordinators.asset_select

import com.gemwallet.android.application.asset_select.coordinators.ClearRecentAssets
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.model.RecentType
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first

class ClearRecentAssetsImpl(
    private val sessionRepository: SessionRepository,
    private val assetsRepository: AssetsRepository,
) : ClearRecentAssets {
    override suspend fun invoke(types: List<RecentType>) {
        val wallet = sessionRepository.session().filterNotNull().first().wallet
        assetsRepository.clearRecentAssets(wallet.id, types)
    }
}
