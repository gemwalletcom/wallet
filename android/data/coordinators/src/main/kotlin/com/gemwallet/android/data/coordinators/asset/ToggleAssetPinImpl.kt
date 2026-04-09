package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.ToggleAssetPin
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.wallet.core.primitives.AssetId

class ToggleAssetPinImpl(
    private val sessionRepository: SessionRepository,
    private val assetsRepository: AssetsRepository,
) : ToggleAssetPin {

    override suspend fun invoke(assetId: AssetId) {
        val session = sessionRepository.session().value ?: return
        assetsRepository.togglePin(session.wallet.id, assetId)
    }
}
