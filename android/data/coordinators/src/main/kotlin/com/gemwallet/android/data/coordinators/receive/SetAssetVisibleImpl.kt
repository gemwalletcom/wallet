package com.gemwallet.android.data.coordinators.receive

import com.gemwallet.android.application.receive.coordinators.SetAssetVisible
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.getAccount
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.firstOrNull

class SetAssetVisibleImpl(
    private val sessionRepository: SessionRepository,
    private val assetsRepository: AssetsRepository,
) : SetAssetVisible {

    override suspend fun invoke(assetId: AssetId) {
        val session = sessionRepository.session().firstOrNull() ?: return
        val account = session.wallet.getAccount(assetId.chain) ?: return
        assetsRepository.switchVisibility(session.wallet.id, account, assetId, true)
    }
}
