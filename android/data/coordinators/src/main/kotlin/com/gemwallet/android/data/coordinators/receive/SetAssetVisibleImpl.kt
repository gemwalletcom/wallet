package com.gemwallet.android.data.coordinators.receive

import com.gemwallet.android.application.assets.coordinators.EnableAsset
import com.gemwallet.android.application.receive.coordinators.SetAssetVisible
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.getAccount
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.firstOrNull

class SetAssetVisibleImpl(
    private val sessionRepository: SessionRepository,
    private val enableAsset: EnableAsset,
) : SetAssetVisible {

    override suspend fun invoke(assetId: AssetId) {
        val session = sessionRepository.session().firstOrNull() ?: return
        session.wallet.getAccount(assetId.chain) ?: return
        enableAsset(session.wallet.id, assetId)
    }
}
