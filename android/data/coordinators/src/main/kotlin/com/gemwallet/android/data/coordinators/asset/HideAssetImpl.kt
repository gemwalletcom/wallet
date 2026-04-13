package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.HideAsset
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.getAccount
import com.wallet.core.primitives.AssetId

class HideAssetImpl(
    private val sessionRepository: SessionRepository,
    private val assetsRepository: AssetsRepository,
) : HideAsset {

    override suspend fun invoke(assetId: AssetId) {
        val session = sessionRepository.session().value ?: return
        val account = session.wallet.getAccount(assetId.chain) ?: return
        assetsRepository.switchVisibility(session.wallet.id, account, assetId, false)
    }
}
