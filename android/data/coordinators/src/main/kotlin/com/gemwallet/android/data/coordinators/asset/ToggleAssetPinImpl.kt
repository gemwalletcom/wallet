package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.ToggleAssetPin
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.firstOrNull
import uniffi.gemstone.GemBalanceService

class ToggleAssetPinImpl(
    private val sessionRepository: SessionRepository,
    private val assetsRepository: AssetsRepository,
    private val balanceService: GemBalanceService,
) : ToggleAssetPin {

    override suspend fun invoke(assetId: AssetId) {
        val session = sessionRepository.session().value ?: return
        val pinned = assetsRepository.getAssetInfo(assetId).firstOrNull()?.metadata?.isPinned == true
        balanceService.pinAsset(
            walletId = session.wallet.id.id,
            assetId = assetId.toIdentifier(),
            pinned = !pinned,
            currency = sessionRepository.getCurrentCurrency().toJson(),
        )
    }
}
